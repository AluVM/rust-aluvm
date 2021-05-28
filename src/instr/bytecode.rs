// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use alloc::vec::Vec;
use bitcoin_hashes::Hash;
use core::ops::RangeInclusive;

use super::instr::*;
use crate::encoding::{Cursor, CursorError, Read, Write};
use crate::instr::{
    ArithmeticOp, BitwiseOp, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp, MoveOp, NOp,
    PutOp, Secp256k1Op,
};
use crate::reg::{Reg, RegBlock, Value};
use crate::{Blob, Instr, InstructionSet, LibHash, LibSite};

/// Errors decoding bytecode
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(doc_comments)]
#[cfg_attr(feature = "std", derive(Error))]
#[allow(clippy::branches_sharing_code)]
pub enum DecodeError {
    /// Cursor error
    #[display(inner)]
    #[from]
    Cursor(CursorError),

    /// Instruction `{0}` is reserved for the future use and currently is not
    /// supported
    ReservedInstruction(u8),
}

/// Errors encoding instructions
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(doc_comments)]
#[cfg_attr(feature = "std", derive(Error))]
#[allow(clippy::branches_sharing_code)]
pub enum EncodeError {
    /// Number of instructions ({0}) exceeds limit of 2^16
    TooManyInstructions(usize),

    /// Cursor error
    #[display(inner)]
    #[from]
    Cursor(CursorError),
}

/// Decodes library from bytecode string
pub fn disassemble<E>(bytecode: impl AsRef<[u8]>) -> Result<Vec<Instr<E>>, DecodeError>
where
    E: InstructionSet,
{
    let bytecode = bytecode.as_ref();
    let len = bytecode.len();
    if len > u16::MAX as usize {
        return Err(DecodeError::Cursor(CursorError::OutOfBoundaries(len)));
    }
    let mut code = Vec::with_capacity(len);
    let mut reader = Cursor::with(bytecode);
    while !reader.is_end() {
        code.push(Instr::read(&mut reader)?);
    }
    Ok(code)
}

/// Encodes library as bytecode
pub fn compile<E, I>(code: I) -> Result<Blob, EncodeError>
where
    E: InstructionSet,
    I: IntoIterator,
    <I as IntoIterator>::Item: InstructionSet,
{
    let mut bytecode = Blob::default();
    let mut writer = Cursor::with(&mut bytecode.bytes[..]);
    for instr in code.into_iter() {
        instr.write(&mut writer)?;
    }
    bytecode.len = writer.pos();
    Ok(bytecode)
}

/// Non-failiable byte encoding for the instruction set. We can't use `io` since
/// (1) we are no_std, (2) it operates data with unlimited length (while we are
/// bound by u16), (3) it provides too many fails in situations when we can't
/// fail because of `u16`-bounding and exclusive in-memory encoding handling.
pub trait Bytecode {
    /// Returns number of bytes which instruction and its argument occupies
    fn byte_count(&self) -> u16;

    /// Returns range of instruction btecodes covered by a set of operations
    fn instr_range() -> RangeInclusive<u8>;

    /// Returns byte representing instruction code (without its arguments)
    fn instr_byte(&self) -> u8;

    /// Writes the instruction as bytecode
    fn write<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        writer.write_u8(self.instr_byte())?;
        self.write_args(writer)
    }

    /// Writes instruction arguments as bytecode, omitting instruction code byte
    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>;

    /// Reads the instruction from bytecode
    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized,
        R: Read,
        DecodeError: From<<R as Read>::Error>;
}

impl<Extension> Bytecode for Instr<Extension>
where
    Extension: InstructionSet,
{
    fn byte_count(&self) -> u16 {
        match self {
            Instr::ControlFlow(instr) => instr.byte_count(),
            Instr::Put(instr) => instr.byte_count(),
            Instr::Move(instr) => instr.byte_count(),
            Instr::Cmp(instr) => instr.byte_count(),
            Instr::Arithmetic(instr) => instr.byte_count(),
            Instr::Bitwise(instr) => instr.byte_count(),
            Instr::Bytes(instr) => instr.byte_count(),
            Instr::Digest(instr) => instr.byte_count(),
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.byte_count(),
            #[cfg(feature = "curve25519")]
            Instr::Curve25519(instr) => instr.byte_count(),
            Instr::ExtensionCodes(instr) => instr.byte_count(),
            Instr::Nop => 1,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        0..=u8::MAX
    }

    fn instr_byte(&self) -> u8 {
        match self {
            Instr::ControlFlow(instr) => instr.instr_byte(),
            Instr::Put(instr) => instr.instr_byte(),
            Instr::Move(instr) => instr.instr_byte(),
            Instr::Cmp(instr) => instr.instr_byte(),
            Instr::Arithmetic(instr) => instr.instr_byte(),
            Instr::Bitwise(instr) => instr.instr_byte(),
            Instr::Bytes(instr) => instr.instr_byte(),
            Instr::Digest(instr) => instr.instr_byte(),
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.instr_byte(),
            #[cfg(feature = "curve25519")]
            Instr::Curve25519(instr) => instr.instr_byte(),
            Instr::ExtensionCodes(instr) => instr.instr_byte(),
            Instr::Nop => 1,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            Instr::ControlFlow(instr) => instr.write_args(writer),
            Instr::Put(instr) => instr.write_args(writer),
            Instr::Move(instr) => instr.write_args(writer),
            Instr::Cmp(instr) => instr.write_args(writer),
            Instr::Arithmetic(instr) => instr.write_args(writer),
            Instr::Bitwise(instr) => instr.write_args(writer),
            Instr::Bytes(instr) => instr.write_args(writer),
            Instr::Digest(instr) => instr.write_args(writer),
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.write_args(writer),
            #[cfg(feature = "curve25519")]
            Instr::Curve25519(instr) => instr.write_args(writer),
            Instr::ExtensionCodes(instr) => instr.write_args(writer),
            Instr::Nop => Ok(()),
        }
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        let instr = reader.peek_u8()?;
        Ok(match instr {
            instr if ControlFlowOp::instr_range().contains(&instr) => {
                Instr::ControlFlow(ControlFlowOp::read(reader)?)
            }
            instr if PutOp::instr_range().contains(&instr) => Instr::Put(PutOp::read(reader)?),
            instr if MoveOp::instr_range().contains(&instr) => Instr::Move(MoveOp::read(reader)?),
            instr if CmpOp::instr_range().contains(&instr) => Instr::Cmp(CmpOp::read(reader)?),
            instr if ArithmeticOp::instr_range().contains(&instr) => {
                Instr::Arithmetic(ArithmeticOp::read(reader)?)
            }
            instr if BitwiseOp::instr_range().contains(&instr) => {
                Instr::Bitwise(BitwiseOp::read(reader)?)
            }
            instr if BytesOp::instr_range().contains(&instr) => {
                Instr::Bytes(BytesOp::read(reader)?)
            }
            instr if DigestOp::instr_range().contains(&instr) => {
                Instr::Digest(DigestOp::read(reader)?)
            }
            #[cfg(feature = "secp256k1")]
            instr if Secp256k1Op::instr_range().contains(&instr) => {
                Instr::Secp256k1(Secp256k1Op::read(reader)?)
            }
            #[cfg(feature = "curve25519")]
            instr if Curve25519Op::instr_range().contains(&instr) => {
                Instr::Curve25519(Curve25519Op::read(reader)?)
            }
            INSTR_EXT_FROM..=INSTR_EXT_TO => Instr::ExtensionCodes(Extension::read(reader)?),
            INSTR_RESV_FROM..=INSTR_RESV_TO => {
                let _ = reader.read_u8()?;
                return Err(DecodeError::ReservedInstruction(instr));
            }
            INSTR_NOP => Instr::Nop,
            x => unreachable!("unable to classify instruction {:#010b}", x),
        })
    }
}

impl Bytecode for ControlFlowOp {
    fn byte_count(&self) -> u16 {
        match self {
            ControlFlowOp::Fail | ControlFlowOp::Succ => 1,
            ControlFlowOp::Jmp(_) | ControlFlowOp::Jif(_) => 3,
            ControlFlowOp::Routine(_) => 3,
            ControlFlowOp::Call(_) => 3 + 32,
            ControlFlowOp::Exec(_) => 3 + 32,
            ControlFlowOp::Ret => 1,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_FAIL..=INSTR_RET
    }

    fn instr_byte(&self) -> u8 {
        match self {
            ControlFlowOp::Fail => INSTR_FAIL,
            ControlFlowOp::Succ => INSTR_SUCC,
            ControlFlowOp::Jmp(_) => INSTR_JMP,
            ControlFlowOp::Jif(_) => INSTR_JIF,
            ControlFlowOp::Routine(_) => INSTR_ROUTINE,
            ControlFlowOp::Call(_) => INSTR_CALL,
            ControlFlowOp::Exec(_) => INSTR_EXEC,
            ControlFlowOp::Ret => INSTR_RET,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            ControlFlowOp::Fail => {}
            ControlFlowOp::Succ => {}
            ControlFlowOp::Jmp(pos) | ControlFlowOp::Jif(pos) | ControlFlowOp::Routine(pos) => {
                writer.write_u16(*pos)?
            }
            ControlFlowOp::Call(lib_site) | ControlFlowOp::Exec(lib_site) => {
                writer.write_u16(lib_site.pos)?;
                writer.write_bytes32(lib_site.lib.into_inner())?;
            }
            ControlFlowOp::Ret => {}
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_FAIL => Self::Fail,
            INSTR_SUCC => Self::Succ,
            INSTR_JMP => Self::Jmp(reader.read_u16()?),
            INSTR_JIF => Self::Jif(reader.read_u16()?),
            INSTR_ROUTINE => Self::Routine(reader.read_u16()?),
            INSTR_CALL => Self::Call(LibSite::with(
                reader.read_u16()?,
                LibHash::from_inner(reader.read_bytes32()?),
            )),
            INSTR_EXEC => Self::Exec(LibSite::with(
                reader.read_u16()?,
                LibHash::from_inner(reader.read_bytes32()?),
            )),
            INSTR_RET => Self::Ret,
            x => unreachable!(
                "instruction {:#010b} classified as control flow operation",
                x
            ),
        })
    }
}

impl Bytecode for PutOp {
    fn byte_count(&self) -> u16 {
        match self {
            PutOp::ZeroA(_, _) | PutOp::ZeroR(_, _) | PutOp::ClA(_, _) | PutOp::ClR(_, _) => 2,
            PutOp::PutA(reg, _, Value { len, .. }) | PutOp::PutIfA(reg, _, Value { len, .. }) => {
                2u16.saturating_add(reg.bits().map(|bits| bits / 8).unwrap_or(*len))
            }
            PutOp::PutR(reg, _, Value { len, .. }) | PutOp::PutIfR(reg, _, Value { len, .. }) => {
                2u16.saturating_add(reg.bits().map(|bits| bits / 8).unwrap_or(*len))
            }
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_ZEROA..=INSTR_PUTIFR
    }

    fn instr_byte(&self) -> u8 {
        match self {
            PutOp::ZeroA(_, _) => INSTR_ZEROA,
            PutOp::ZeroR(_, _) => INSTR_ZEROR,
            PutOp::ClA(_, _) => INSTR_CLA,
            PutOp::ClR(_, _) => INSTR_CLR,
            PutOp::PutA(_, _, _) => INSTR_PUTA,
            PutOp::PutR(_, _, _) => INSTR_PUTR,
            PutOp::PutIfA(_, _, _) => INSTR_PUTIFA,
            PutOp::PutIfR(_, _, _) => INSTR_PUTIFR,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            PutOp::ZeroA(reg, reg32) | PutOp::ClA(reg, reg32) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
            }
            PutOp::ZeroR(reg, reg32) | PutOp::ClR(reg, reg32) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
            }
            PutOp::PutA(reg, reg32, val) | PutOp::PutIfA(reg, reg32, val) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
                writer.write_value(Reg::A(*reg), val)?;
            }
            PutOp::PutR(reg, reg32, val) | PutOp::PutIfR(reg, reg32, val) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
                writer.write_value(Reg::R(*reg), val)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_ZEROA => Self::ZeroA(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_ZEROR => Self::ZeroR(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_CLA => Self::ClA(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_CLR => Self::ClR(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_PUTA => {
                let reg = reader.read_u3()?.into();
                Self::PutA(
                    reg,
                    reader.read_u5()?.into(),
                    reader.read_value(Reg::A(reg))?,
                )
            }
            INSTR_PUTR => {
                let reg = reader.read_u3()?.into();
                Self::PutR(
                    reg,
                    reader.read_u5()?.into(),
                    reader.read_value(Reg::R(reg))?,
                )
            }
            INSTR_PUTIFA => {
                let reg = reader.read_u3()?.into();
                Self::PutIfA(
                    reg,
                    reader.read_u5()?.into(),
                    reader.read_value(Reg::A(reg))?,
                )
            }
            INSTR_PUTIFR => {
                let reg = reader.read_u3()?.into();
                Self::PutIfR(
                    reg,
                    reader.read_u5()?.into(),
                    reader.read_value(Reg::R(reg))?,
                )
            }
            x => unreachable!("instruction {:#010b} classified as put operation", x),
        })
    }
}

impl Bytecode for MoveOp {
    fn byte_count(&self) -> u16 {
        match self {
            MoveOp::SwpA(_, _, _, _) | MoveOp::SwpR(_, _, _, _) | MoveOp::SwpAR(_, _, _, _) => 3,
            MoveOp::AMov(_, _, _) => 2,
            MoveOp::MovA(_, _, _, _)
            | MoveOp::MovR(_, _, _, _)
            | MoveOp::MovAR(_, _, _, _)
            | MoveOp::MovRA(_, _, _, _) => 3,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_SWPA..=INSTR_MOVRA
    }

    fn instr_byte(&self) -> u8 {
        match self {
            MoveOp::SwpA(_, _, _, _) => INSTR_SWPA,
            MoveOp::SwpR(_, _, _, _) => INSTR_SWPR,
            MoveOp::SwpAR(_, _, _, _) => INSTR_SWPAR,
            MoveOp::AMov(_, _, _) => INSTR_AMOV,
            MoveOp::MovA(_, _, _, _) => INSTR_MOVA,
            MoveOp::MovR(_, _, _, _) => INSTR_MOVR,
            MoveOp::MovAR(_, _, _, _) => INSTR_MOVAR,
            MoveOp::MovRA(_, _, _, _) => INSTR_MOVRA,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            MoveOp::SwpA(reg1, idx1, reg2, idx2) | MoveOp::MovA(reg1, idx1, reg2, idx2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            MoveOp::SwpR(reg1, idx1, reg2, idx2) | MoveOp::MovR(reg1, idx1, reg2, idx2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            MoveOp::SwpAR(reg1, idx1, reg2, idx2) | MoveOp::MovAR(reg1, idx1, reg2, idx2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            MoveOp::MovRA(reg1, idx1, reg2, idx2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            MoveOp::AMov(reg1, reg2, nt) => {
                writer.write_u3(reg1)?;
                writer.write_u3(reg2)?;
                writer.write_u2(nt)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_SWPA => Self::SwpA(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_SWPR => Self::SwpR(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_SWPAR => Self::SwpAR(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_MOVA => Self::MovA(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_MOVR => Self::MovR(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_MOVAR => Self::MovAR(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_MOVRA => Self::MovRA(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_AMOV => Self::AMov(
                reader.read_u3()?.into(),
                reader.read_u3()?.into(),
                reader.read_u2()?.into(),
            ),
            x => unreachable!("instruction {:#010b} classified as move operation", x),
        })
    }
}

impl Bytecode for CmpOp {
    fn byte_count(&self) -> u16 {
        match self {
            CmpOp::GtA(_, _, _, _)
            | CmpOp::LtA(_, _, _, _)
            | CmpOp::GtR(_, _, _, _)
            | CmpOp::LtR(_, _, _, _)
            | CmpOp::EqA(_, _, _, _)
            | CmpOp::EqR(_, _, _, _) => 3,
            CmpOp::Len(_, _) | CmpOp::Cnt(_, _) => 2,
            CmpOp::St2A | CmpOp::A2St => 1,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_GT..=INSTR_A2ST
    }

    fn instr_byte(&self) -> u8 {
        match self {
            CmpOp::GtA(_, _, _, _) | CmpOp::GtR(_, _, _, _) => INSTR_GT,
            CmpOp::LtA(_, _, _, _) | CmpOp::LtR(_, _, _, _) => INSTR_LT,
            CmpOp::EqA(_, _, _, _) => INSTR_EQA,
            CmpOp::EqR(_, _, _, _) => INSTR_EQR,
            CmpOp::Len(_, _) => INSTR_LEN,
            CmpOp::Cnt(_, _) => INSTR_CNT,
            CmpOp::St2A => INSTR_ST2A,
            CmpOp::A2St => INSTR_A2ST,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            CmpOp::GtA(num_type, reg, idx1, idx2) | CmpOp::LtA(num_type, reg, idx1, idx2) => {
                writer.write_u4(Reg::A(*reg))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u2(num_type)?;
            }
            CmpOp::GtR(reg1, idx1, reg2, idx2) | CmpOp::LtR(reg1, idx1, reg2, idx2) => {
                writer.write_u4(Reg::R(*reg1))?;
                writer.write_u4(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            CmpOp::EqA(reg1, idx1, reg2, idx2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            CmpOp::EqR(reg1, idx1, reg2, idx2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(idx1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(idx2)?;
            }
            CmpOp::Len(reg, idx) | CmpOp::Cnt(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            CmpOp::St2A => {}
            CmpOp::A2St => {}
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_GT => {
                if reader.read_bool()? {
                    Self::GtR(
                        reader.read_u3()?.into(),
                        reader.read_u4()?.into(),
                        reader.read_u3()?.into(),
                        reader.read_u5()?.into(),
                    )
                } else {
                    let reg = reader.read_u3()?.into();
                    let idx1 = reader.read_u5()?.into();
                    let idx2 = reader.read_u5()?.into();
                    let num_type = reader.read_u2()?.into();
                    Self::GtA(num_type, reg, idx1, idx2)
                }
            }
            INSTR_LT => {
                if reader.read_bool()? {
                    Self::LtR(
                        reader.read_u3()?.into(),
                        reader.read_u4()?.into(),
                        reader.read_u3()?.into(),
                        reader.read_u5()?.into(),
                    )
                } else {
                    let reg = reader.read_u3()?.into();
                    let idx1 = reader.read_u5()?.into();
                    let idx2 = reader.read_u5()?.into();
                    let num_type = reader.read_u2()?.into();
                    Self::LtA(num_type, reg, idx1, idx2)
                }
            }
            INSTR_EQA => Self::EqA(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_EQR => Self::EqR(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_LEN => Self::Len(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_CNT => Self::Cnt(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_ST2A => Self::St2A,
            INSTR_A2ST => Self::A2St,
            x => unreachable!("instruction {:#010b} classified as comparison operation", x),
        })
    }
}

impl Bytecode for ArithmeticOp {
    fn byte_count(&self) -> u16 {
        match self {
            ArithmeticOp::Neg(_, _) => 2,
            ArithmeticOp::Stp(_, _, _, _, _) => 3,
            ArithmeticOp::Add(_, _, _, _)
            | ArithmeticOp::Sub(_, _, _, _)
            | ArithmeticOp::Mul(_, _, _, _)
            | ArithmeticOp::Div(_, _, _, _)
            | ArithmeticOp::Rem(_, _, _, _) => 3,
            ArithmeticOp::Abs(_, _) => 2,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_NEG..=INSTR_ABS
    }

    fn instr_byte(&self) -> u8 {
        match self {
            ArithmeticOp::Neg(_, _) => INSTR_NEG,
            ArithmeticOp::Stp(_, _, _, _, _) => INSTR_STP,
            ArithmeticOp::Add(_, _, _, _) => INSTR_ADD,
            ArithmeticOp::Sub(_, _, _, _) => INSTR_SUB,
            ArithmeticOp::Mul(_, _, _, _) => INSTR_MUL,
            ArithmeticOp::Div(_, _, _, _) => INSTR_DIV,
            ArithmeticOp::Rem(_, _, _, _) => INSTR_REM,
            ArithmeticOp::Abs(_, _) => INSTR_ABS,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            ArithmeticOp::Neg(reg, idx) | ArithmeticOp::Abs(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            ArithmeticOp::Stp(op, ar, reg, idx, step) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
                writer.write_u4(*step)?;
                writer.write_bool(op.into())?;
                writer.write_u3(ar)?;
            }
            ArithmeticOp::Add(ar, reg, src1, src2)
            | ArithmeticOp::Sub(ar, reg, src1, src2)
            | ArithmeticOp::Mul(ar, reg, src1, src2)
            | ArithmeticOp::Div(ar, reg, src1, src2)
            | ArithmeticOp::Rem(ar, reg, src1, src2) => {
                writer.write_u3(reg)?;
                writer.write_u5(src1)?;
                writer.write_u5(src2)?;
                writer.write_u3(ar)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_NEG => Self::Neg(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_STP => {
                let reg = reader.read_u3()?.into();
                let idx = reader.read_u5()?.into();
                let step = reader.read_u4()?;
                let op = reader.read_bool()?.into();
                let ar = reader.read_u3()?.into();
                Self::Stp(op, ar, reg, idx, step)
            }
            INSTR_ADD => {
                let reg = reader.read_u3()?.into();
                let src1 = reader.read_u5()?.into();
                let src2 = reader.read_u5()?.into();
                let ar = reader.read_u3()?.into();
                Self::Add(ar, reg, src1, src2)
            }
            INSTR_SUB => {
                let reg = reader.read_u3()?.into();
                let src1 = reader.read_u5()?.into();
                let src2 = reader.read_u5()?.into();
                let ar = reader.read_u3()?.into();
                Self::Sub(ar, reg, src1, src2)
            }
            INSTR_MUL => {
                let reg = reader.read_u3()?.into();
                let src1 = reader.read_u5()?.into();
                let src2 = reader.read_u5()?.into();
                let ar = reader.read_u3()?.into();
                Self::Mul(ar, reg, src1, src2)
            }
            INSTR_DIV => {
                let reg = reader.read_u3()?.into();
                let src1 = reader.read_u5()?.into();
                let src2 = reader.read_u5()?.into();
                let ar = reader.read_u3()?.into();
                Self::Div(ar, reg, src1, src2)
            }
            INSTR_REM => {
                let reg = reader.read_u3()?.into();
                let src1 = reader.read_u5()?.into();
                let src2 = reader.read_u5()?.into();
                let ar = reader.read_u3()?.into();
                Self::Rem(ar, reg, src1, src2)
            }
            INSTR_ABS => Self::Abs(reader.read_u3()?.into(), reader.read_u5()?.into()),
            x => unreachable!("instruction {:#010b} classified as arithmetic operation", x),
        })
    }
}

impl Bytecode for BitwiseOp {
    fn byte_count(&self) -> u16 {
        match self {
            BitwiseOp::And(_, _, _, _) | BitwiseOp::Or(_, _, _, _) | BitwiseOp::Xor(_, _, _, _) => {
                3
            }
            BitwiseOp::Not(_, _) => 2,
            BitwiseOp::Shl(_, _, _, _)
            | BitwiseOp::Shr(_, _, _, _)
            | BitwiseOp::Scl(_, _, _, _)
            | BitwiseOp::Scr(_, _, _, _) => 3,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_AND..=INSTR_SCR
    }

    fn instr_byte(&self) -> u8 {
        match self {
            BitwiseOp::And(_, _, _, _) => INSTR_AND,
            BitwiseOp::Or(_, _, _, _) => INSTR_OR,
            BitwiseOp::Xor(_, _, _, _) => INSTR_XOR,
            BitwiseOp::Not(_, _) => INSTR_NOT,
            BitwiseOp::Shl(_, _, _, _) => INSTR_SHL,
            BitwiseOp::Shr(_, _, _, _) => INSTR_SHR,
            BitwiseOp::Scl(_, _, _, _) => INSTR_SCL,
            BitwiseOp::Scr(_, _, _, _) => INSTR_SCR,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            BitwiseOp::And(reg, idx1, idx2, idx3)
            | BitwiseOp::Or(reg, idx1, idx2, idx3)
            | BitwiseOp::Xor(reg, idx1, idx2, idx3)
            | BitwiseOp::Shl(reg, idx1, idx2, idx3)
            | BitwiseOp::Shr(reg, idx1, idx2, idx3)
            | BitwiseOp::Scl(reg, idx1, idx2, idx3)
            | BitwiseOp::Scr(reg, idx1, idx2, idx3) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(idx3)?;
            }
            BitwiseOp::Not(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        let instr = reader.read_u8()?;
        if instr == INSTR_NOT {
            return Ok(Self::Not(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
            ));
        }
        let reg = reader.read_u3()?.into();
        let src1 = reader.read_u5()?.into();
        let src2 = reader.read_u5()?.into();
        let dst = reader.read_u3()?.into();

        Ok(match instr {
            INSTR_AND => Self::And(reg, src1, src2, dst),
            INSTR_OR => Self::Or(reg, src1, src2, dst),
            INSTR_XOR => Self::Xor(reg, src1, src2, dst),
            INSTR_SHL => Self::Shl(reg, src1, src2, dst),
            INSTR_SHR => Self::Shr(reg, src1, src2, dst),
            INSTR_SCL => Self::Scl(reg, src1, src2, dst),
            INSTR_SCR => Self::Scr(reg, src1, src2, dst),
            x => unreachable!("instruction {:#010b} classified as bitwise operation", x),
        })
    }
}

impl Bytecode for BytesOp {
    fn byte_count(&self) -> u16 {
        match self {
            BytesOp::Put(_, Blob { len, .. }) => 4u16.saturating_add(*len),
            BytesOp::Mov(_, _) | BytesOp::Swp(_, _) => 3,
            BytesOp::Fill(_, _, _, _) => 7,
            BytesOp::LenS(_) => 2,
            BytesOp::Count(_, _) => 3,
            BytesOp::Cmp(_, _) => 3,
            BytesOp::Comm(_, _) => 3,
            BytesOp::Find(_, _) => 3,
            BytesOp::Extr(_, _, _, _) | BytesOp::Inj(_, _, _, _) => 4,
            BytesOp::Join(_, _, _) => 4,
            BytesOp::Split(_, _, _, _) => 6,
            BytesOp::Ins(_, _, _) | BytesOp::Del(_, _, _) => 5,
            BytesOp::Transl(_, _, _, _) => 7,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_PUT..=INSTR_TRANSL
    }

    fn instr_byte(&self) -> u8 {
        match self {
            BytesOp::Put(_, _) => INSTR_PUT,
            BytesOp::Mov(_, _) => INSTR_MOV,
            BytesOp::Swp(_, _) => INSTR_SWP,
            BytesOp::Fill(_, _, _, _) => INSTR_FILL,
            BytesOp::LenS(_) => INSTR_LENS,
            BytesOp::Count(_, _) => INSTR_COUNT,
            BytesOp::Cmp(_, _) => INSTR_CMP,
            BytesOp::Comm(_, _) => INSTR_COMM,
            BytesOp::Find(_, _) => INSTR_FIND,
            BytesOp::Extr(_, _, _, _) => INSTR_EXTR,
            BytesOp::Inj(_, _, _, _) => INSTR_INJ,
            BytesOp::Join(_, _, _) => INSTR_JOIN,
            BytesOp::Split(_, _, _, _) => INSTR_SPLIT,
            BytesOp::Ins(_, _, _) => INSTR_DEL,
            BytesOp::Del(_, _, _) => INSTR_INS,
            BytesOp::Transl(_, _, _, _) => INSTR_TRANSL,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            BytesOp::Put(reg, blob) => {
                writer.write_u8(*reg)?;
                writer.write_slice(&blob.bytes[..blob.len as usize])?;
            }
            BytesOp::Mov(reg1, reg2)
            | BytesOp::Swp(reg1, reg2)
            | BytesOp::Count(reg1, reg2)
            | BytesOp::Cmp(reg1, reg2)
            | BytesOp::Comm(reg1, reg2)
            | BytesOp::Find(reg1, reg2) => {
                writer.write_u8(*reg1)?;
                writer.write_u8(*reg2)?;
            }
            BytesOp::Fill(reg, from, to, value) => {
                writer.write_u8(*reg)?;
                writer.write_u16(*from)?;
                writer.write_u16(*to)?;
                writer.write_u8(*value)?;
            }
            BytesOp::LenS(reg) => {
                writer.write_u8(*reg)?;
            }
            BytesOp::Extr(src, offset, dst, index) | BytesOp::Inj(src, offset, dst, index) => {
                writer.write_u5(src)?;
                writer.write_u5(offset)?;
                writer.write_bool(*dst == RegBlock::A)?;
                writer.write_u5(index)?;
            }
            BytesOp::Join(src1, src2, dst) => {
                writer.write_u8(*src1)?;
                writer.write_u8(*src2)?;
                writer.write_u8(*dst)?;
            }
            BytesOp::Split(src, pos, dst1, dst2) => {
                writer.write_u8(*src)?;
                writer.write_u16(*pos)?;
                writer.write_u8(*dst1)?;
                writer.write_u8(*dst2)?;
            }
            BytesOp::Ins(src, dst, offset) => {
                writer.write_u8(*src)?;
                writer.write_u8(*dst)?;
                writer.write_u16(*offset)?;
            }
            BytesOp::Del(reg, from, to) => {
                writer.write_u8(*reg)?;
                writer.write_u16(*from)?;
                writer.write_u16(*to)?;
            }
            BytesOp::Transl(src, from, to, dst) => {
                writer.write_u8(*src)?;
                writer.write_u16(*from)?;
                writer.write_u16(*to)?;
                writer.write_u16(*dst)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_PUT => Self::Put(reader.read_u8()?, Blob::with(reader.read_slice()?)),
            INSTR_MOV => Self::Mov(reader.read_u8()?, reader.read_u8()?),
            INSTR_SWP => Self::Swp(reader.read_u8()?, reader.read_u8()?),
            INSTR_FILL => Self::Fill(
                reader.read_u8()?,
                reader.read_u16()?,
                reader.read_u16()?,
                reader.read_u8()?,
            ),
            INSTR_LENS => Self::LenS(reader.read_u8()?),
            INSTR_COUNT => Self::Count(reader.read_u8()?, reader.read_u8()?),
            INSTR_CMP => Self::Cmp(reader.read_u8()?, reader.read_u8()?),
            INSTR_COMM => Self::Comm(reader.read_u8()?, reader.read_u8()?),
            INSTR_FIND => Self::Find(reader.read_u8()?, reader.read_u8()?),
            INSTR_EXTR => Self::Extr(
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                if reader.read_bool()? {
                    RegBlock::A
                } else {
                    RegBlock::R
                },
                reader.read_u5()?.into(),
            ),
            INSTR_INJ => Self::Inj(
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                if reader.read_bool()? {
                    RegBlock::A
                } else {
                    RegBlock::R
                },
                reader.read_u5()?.into(),
            ),
            INSTR_JOIN => Self::Join(reader.read_u8()?, reader.read_u8()?, reader.read_u8()?),
            INSTR_SPLIT => Self::Split(
                reader.read_u8()?,
                reader.read_u16()?,
                reader.read_u8()?,
                reader.read_u8()?,
            ),
            INSTR_INS => Self::Ins(reader.read_u8()?, reader.read_u8()?, reader.read_u16()?),
            INSTR_DEL => Self::Del(reader.read_u8()?, reader.read_u16()?, reader.read_u16()?),
            INSTR_TRANSL => Self::Transl(
                reader.read_u8()?,
                reader.read_u16()?,
                reader.read_u16()?,
                reader.read_u8()?,
            ),
            x => unreachable!(
                "instruction {:#010b} classified as byte string operation",
                x
            ),
        })
    }
}

impl Bytecode for DigestOp {
    fn byte_count(&self) -> u16 {
        3
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_RIPEMD..=INSTR_HASH5
    }

    fn instr_byte(&self) -> u8 {
        match self {
            DigestOp::Ripemd(_, _) => INSTR_RIPEMD,
            DigestOp::Sha256(_, _) => INSTR_SHA256,
            DigestOp::Sha512(_, _) => INSTR_SHA512,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            DigestOp::Ripemd(src, dst)
            | DigestOp::Sha256(src, dst)
            | DigestOp::Sha512(src, dst) => {
                writer.write_u5(src)?;
                writer.write_u3(dst)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        let instr = reader.read_u8()?;
        let src = reader.read_u5()?.into();
        let dst = reader.read_u3()?.into();

        Ok(match instr {
            INSTR_RIPEMD => Self::Ripemd(src, dst),
            INSTR_SHA256 => Self::Sha256(src, dst),
            INSTR_SHA512 => Self::Sha512(src, dst),
            INSTR_HASH1..=INSTR_HASH5 => return Err(DecodeError::ReservedInstruction(instr)),
            x => unreachable!("instruction {:#010b} classified as digest operation", x),
        })
    }
}

impl Bytecode for Secp256k1Op {
    fn byte_count(&self) -> u16 {
        match self {
            Secp256k1Op::Gen(_, _) => 2,
            Secp256k1Op::Mul(_, _, _, _) => 3,
            Secp256k1Op::Add(_, _) => 2,
            Secp256k1Op::Neg(_, _) => 2,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_SECP_GEN..=INSTR_SECP_NEG
    }

    fn instr_byte(&self) -> u8 {
        match self {
            Secp256k1Op::Gen(_, _) => INSTR_SECP_GEN,
            Secp256k1Op::Mul(_, _, _, _) => INSTR_SECP_MUL,
            Secp256k1Op::Add(_, _) => INSTR_SECP_ADD,
            Secp256k1Op::Neg(_, _) => INSTR_SECP_NEG,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            Secp256k1Op::Gen(src, dst) => {
                writer.write_u5(src)?;
                writer.write_u3(dst)?;
            }
            Secp256k1Op::Mul(reg, scal, src, dst) => {
                writer.write_bool(*reg == RegBlock::A)?;
                writer.write_u5(scal)?;
                writer.write_u5(src)?;
                writer.write_u5(dst)?;
            }
            Secp256k1Op::Add(src, srcdst) => {
                writer.write_u5(src)?;
                writer.write_u3(srcdst)?;
            }
            Secp256k1Op::Neg(src, dst) => {
                writer.write_u5(src)?;
                writer.write_u3(dst)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_SECP_GEN => Self::Gen(reader.read_u5()?.into(), reader.read_u3()?.into()),
            INSTR_SECP_MUL => Self::Mul(
                if reader.read_bool()? {
                    RegBlock::A
                } else {
                    RegBlock::R
                },
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_SECP_ADD => Self::Add(reader.read_u5()?.into(), reader.read_u3()?.into()),
            INSTR_SECP_NEG => Self::Neg(reader.read_u5()?.into(), reader.read_u3()?.into()),
            x => unreachable!(
                "instruction {:#010b} classified as Secp256k1 curve operation",
                x
            ),
        })
    }
}

impl Bytecode for Curve25519Op {
    fn byte_count(&self) -> u16 {
        match self {
            Curve25519Op::Gen(_, _) => 2,
            Curve25519Op::Mul(_, _, _, _) => 3,
            Curve25519Op::Add(_, _, _, _) => 3,
            Curve25519Op::Neg(_, _) => 2,
        }
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_ED_GEN..=INSTR_ED_NEG
    }

    fn instr_byte(&self) -> u8 {
        match self {
            Curve25519Op::Gen(_, _) => INSTR_ED_GEN,
            Curve25519Op::Mul(_, _, _, _) => INSTR_ED_MUL,
            Curve25519Op::Add(_, _, _, _) => INSTR_ED_ADD,
            Curve25519Op::Neg(_, _) => INSTR_ED_NEG,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            Curve25519Op::Gen(src, dst) => {
                writer.write_u5(src)?;
                writer.write_u3(dst)?;
            }
            Curve25519Op::Mul(reg, scal, src, dst) => {
                writer.write_bool(*reg == RegBlock::A)?;
                writer.write_u5(scal)?;
                writer.write_u5(src)?;
                writer.write_u5(dst)?;
            }
            Curve25519Op::Add(src1, src2, dst, overflow) => {
                writer.write_u5(src1)?;
                writer.write_u5(src2)?;
                writer.write_u5(dst)?;
                writer.write_bool(*overflow)?;
            }
            Curve25519Op::Neg(src, dst) => {
                writer.write_u5(src)?;
                writer.write_u3(dst)?;
            }
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        Ok(match reader.read_u8()? {
            INSTR_ED_GEN => Self::Gen(reader.read_u5()?.into(), reader.read_u3()?.into()),
            INSTR_ED_MUL => Self::Mul(
                if reader.read_bool()? {
                    RegBlock::A
                } else {
                    RegBlock::R
                },
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_ED_ADD => Self::Add(
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_bool()?,
            ),
            INSTR_ED_NEG => Self::Neg(reader.read_u5()?.into(), reader.read_u3()?.into()),
            x => unreachable!("instruction {:#010b} classified as Curve25519 operation", x),
        })
    }
}

impl Bytecode for NOp {
    fn byte_count(&self) -> u16 {
        1
    }

    fn instr_range() -> RangeInclusive<u8> {
        INSTR_NOP..=INSTR_NOP
    }

    fn instr_byte(&self) -> u8 {
        INSTR_NOP
    }

    fn write_args<W>(&self, _writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        let instr = reader.read_u8()?;
        if instr != INSTR_NOP {
            unreachable!("instruction {:#010b} classified as NOP operation", instr)
        }
        Ok(NOp::NOp)
    }
}
