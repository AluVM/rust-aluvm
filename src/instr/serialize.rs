// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://github.com/internet2-org/aluvm-spec>
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use amplify_num::{u1, u2, u3, u5};
use bitcoin_hashes::Hash;

use super::bitcode::*;
use crate::encoding::{Cursor, CursorError, Read, Write};
use crate::instr::{
    ArithmeticOp, BitwiseOp, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp, MoveOp, NOp,
    PutOp, Secp256k1Op,
};
use crate::reg::{RegAR, RegBlockAR, RegisterSet};
use crate::{ByteStr, Instr, InstructionSet, LibHash, LibSite};

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
pub fn compile<E, I>(code: I) -> Result<ByteStr, EncodeError>
where
    E: InstructionSet,
    I: IntoIterator,
    <I as IntoIterator>::Item: InstructionSet,
{
    let mut bytecode = ByteStr::default();
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

    fn instr_range() -> RangeInclusive<u8> { 0..=u8::MAX }

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

    fn instr_range() -> RangeInclusive<u8> { INSTR_FAIL..=INSTR_RET }

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
            x => unreachable!("instruction {:#010b} classified as control flow operation", x),
        })
    }
}

impl Bytecode for PutOp {
    fn byte_count(&self) -> u16 {
        match self {
            PutOp::ClrA(_, _) | PutOp::ClrF(_, _) | PutOp::ClrR(_, _) => 2,
            PutOp::PutA(reg, _, _) | PutOp::PutIfA(reg, _, _) => {
                2u16.saturating_add(reg.bits() / 8)
            }
            PutOp::PutF(reg, _, _) => 2u16.saturating_add(reg.bits() / 8),
            PutOp::PutR(reg, _, _) | PutOp::PutIfR(reg, _, _) => {
                2u16.saturating_add(reg.bits() / 8)
            }
        }
    }

    fn instr_range() -> RangeInclusive<u8> { INSTR_CLRA..=INSTR_PUTIFR }

    fn instr_byte(&self) -> u8 {
        match self {
            PutOp::ClrA(_, _) => INSTR_CLRA,
            PutOp::ClrF(_, _) => INSTR_CLRF,
            PutOp::ClrR(_, _) => INSTR_CLRR,
            PutOp::PutA(_, _, _) => INSTR_PUTA,
            PutOp::PutF(_, _, _) => INSTR_PUTF,
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
            PutOp::ClrA(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            PutOp::ClrF(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            PutOp::ClrR(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            PutOp::PutA(reg, reg32, val) | PutOp::PutIfA(reg, reg32, val) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
                writer.write_value(*reg, *val)?;
            }
            PutOp::PutF(reg, reg32, val) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
                writer.write_value(*reg, *val)?;
            }
            PutOp::PutR(reg, reg32, val) | PutOp::PutIfR(reg, reg32, val) => {
                writer.write_u3(reg)?;
                writer.write_u5(reg32)?;
                writer.write_value(*reg, *val)?;
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
            INSTR_CLRA => Self::ClrA(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_CLRF => Self::ClrF(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_CLRR => Self::ClrR(reader.read_u3()?.into(), reader.read_u5()?.into()),
            INSTR_PUTA => {
                let reg = reader.read_u3()?.into();
                Self::PutA(reg, reader.read_u5()?.into(), reader.read_value(reg)?)
            }
            INSTR_PUTF => {
                let reg = reader.read_u3()?.into();
                Self::PutF(reg, reader.read_u5()?.into(), reader.read_value(reg)?)
            }
            INSTR_PUTR => {
                let reg = reader.read_u3()?.into();
                Self::PutR(reg, reader.read_u5()?.into(), reader.read_value(reg)?)
            }
            INSTR_PUTIFA => {
                let reg = reader.read_u3()?.into();
                Self::PutIfA(reg, reader.read_u5()?.into(), reader.read_value(reg)?)
            }
            INSTR_PUTIFR => {
                let reg = reader.read_u3()?.into();
                Self::PutIfR(reg, reader.read_u5()?.into(), reader.read_value(reg)?)
            }
            x => unreachable!("instruction {:#010b} classified as put operation", x),
        })
    }
}

impl Bytecode for MoveOp {
    fn byte_count(&self) -> u16 { 3 }

    fn instr_range() -> RangeInclusive<u8> { INSTR_MOV..=INSTR_CFA }

    fn instr_byte(&self) -> u8 {
        match self {
            MoveOp::MovA(_, _, _)
            | MoveOp::DupA(_, _, _)
            | MoveOp::SwpA(_, _, _)
            | MoveOp::MovF(_, _, _)
            | MoveOp::DupF(_, _, _)
            | MoveOp::SwpF(_, _, _)
            | MoveOp::MovR(_, _, _)
            | MoveOp::DupR(_, _, _) => INSTR_MOV,
            MoveOp::CpyA(_, _, _, _) => INSTR_CPA,
            MoveOp::CnvA(_, _, _, _) => INSTR_CNA,
            MoveOp::CnvF(_, _, _, _) => INSTR_CNF,
            MoveOp::CpyR(_, _, _, _) => INSTR_CPR,
            MoveOp::SpyAR(_, _, _, _) => INSTR_SPY,
            MoveOp::CnvAF(_, _, _, _) => INSTR_CAF,
            MoveOp::CnvFA(_, _, _, _) => INSTR_CFA,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            MoveOp::MovA(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b000))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::DupA(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b001))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::SwpA(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b010))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::MovF(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b011))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::DupF(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b100))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::SwpF(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b101))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::MovR(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b110))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::DupR(reg, idx1, idx2) => {
                writer.write_u3(u3::with(0b111))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            MoveOp::CpyA(sreg, sidx, dreg, didx) | MoveOp::CnvA(sreg, sidx, dreg, didx) => {
                writer.write_u3(sreg)?;
                writer.write_u5(sidx)?;
                writer.write_u3(dreg)?;
                writer.write_u5(didx)?;
            }
            MoveOp::CnvF(sreg, sidx, dreg, didx) => {
                writer.write_u3(sreg)?;
                writer.write_u5(sidx)?;
                writer.write_u3(dreg)?;
                writer.write_u5(didx)?;
            }
            MoveOp::CpyR(sreg, sidx, dreg, didx) => {
                writer.write_u3(sreg)?;
                writer.write_u5(sidx)?;
                writer.write_u3(dreg)?;
                writer.write_u5(didx)?;
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                writer.write_u3(sreg)?;
                writer.write_u5(sidx)?;
                writer.write_u3(dreg)?;
                writer.write_u5(didx)?;
            }
            MoveOp::CnvAF(sreg, sidx, dreg, didx) => {
                writer.write_u3(sreg)?;
                writer.write_u5(sidx)?;
                writer.write_u3(dreg)?;
                writer.write_u5(didx)?;
            }
            MoveOp::CnvFA(sreg, sidx, dreg, didx) => {
                writer.write_u3(sreg)?;
                writer.write_u5(sidx)?;
                writer.write_u3(dreg)?;
                writer.write_u5(didx)?;
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

        Ok(if instr == INSTR_MOV {
            let code = reader.read_u3()?;
            let idx1 = reader.read_u5()?.into();
            let idx2 = reader.read_u5()?.into();
            let reg = reader.read_u3()?;
            match code.as_u8() {
                0b000 => MoveOp::MovA(reg.into(), idx1, idx2),
                0b001 => MoveOp::DupA(reg.into(), idx1, idx2),
                0b010 => MoveOp::SwpA(reg.into(), idx1, idx2),
                0b011 => MoveOp::MovF(reg.into(), idx1, idx2),
                0b100 => MoveOp::DupF(reg.into(), idx1, idx2),
                0b101 => MoveOp::SwpF(reg.into(), idx1, idx2),
                0b110 => MoveOp::MovR(reg.into(), idx1, idx2),
                0b111 => MoveOp::DupR(reg.into(), idx1, idx2),
                _ => unreachable!(),
            }
        } else {
            let sreg = reader.read_u3()?;
            let sidx = reader.read_u5()?.into();
            let dreg = reader.read_u3()?;
            let didx = reader.read_u5()?.into();
            match instr {
                INSTR_CPA => MoveOp::CpyA(sreg.into(), sidx, dreg.into(), didx),
                INSTR_CNA => MoveOp::CnvA(sreg.into(), sidx, dreg.into(), didx),
                INSTR_CNF => MoveOp::CnvF(sreg.into(), sidx, dreg.into(), didx),
                INSTR_CPR => MoveOp::CpyR(sreg.into(), sidx, dreg.into(), didx),
                INSTR_SPY => MoveOp::SpyAR(sreg.into(), sidx, dreg.into(), didx),
                INSTR_CAF => MoveOp::CnvAF(sreg.into(), sidx, dreg.into(), didx),
                INSTR_CFA => MoveOp::CnvFA(sreg.into(), sidx, dreg.into(), didx),
                x => unreachable!("instruction {:#010b} classified as move operation", x),
            }
        })
    }
}

impl Bytecode for CmpOp {
    fn byte_count(&self) -> u16 {
        match self {
            CmpOp::GtA(_, _, _, _)
            | CmpOp::LtA(_, _, _, _)
            | CmpOp::GtF(_, _, _, _)
            | CmpOp::LtF(_, _, _, _)
            | CmpOp::GtR(_, _, _)
            | CmpOp::LtR(_, _, _)
            | CmpOp::EqA(_, _, _, _)
            | CmpOp::EqF(_, _, _, _)
            | CmpOp::EqR(_, _, _, _) => 3,
            CmpOp::IfZA(_, _) | CmpOp::IfZR(_, _) | CmpOp::IfNA(_, _) | CmpOp::IfNR(_, _) => 2,
            CmpOp::St(_, _, _) => 2,
            CmpOp::StInv => 1,
        }
    }

    fn instr_range() -> RangeInclusive<u8> { INSTR_LGT..=INSTR_STINV }

    fn instr_byte(&self) -> u8 {
        match self {
            CmpOp::GtA(_, _, _, _)
            | CmpOp::LtA(_, _, _, _)
            | CmpOp::GtF(_, _, _, _)
            | CmpOp::LtF(_, _, _, _) => INSTR_LGT,
            CmpOp::GtR(_, _, _)
            | CmpOp::LtR(_, _, _)
            | CmpOp::EqA(_, _, _, _)
            | CmpOp::EqF(_, _, _, _)
            | CmpOp::EqR(_, _, _, _) => INSTR_CMP,
            CmpOp::IfZA(_, _) => INSTR_IFZA,
            CmpOp::IfZR(_, _) => INSTR_IFZR,
            CmpOp::IfNA(_, _) => INSTR_IFNA,
            CmpOp::IfNR(_, _) => INSTR_IFNR,
            CmpOp::St(_, _, _) => INSTR_ST,
            CmpOp::StInv => INSTR_STINV,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            CmpOp::GtA(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b00))?;
                writer.write_u1(flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::LtA(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b01))?;
                writer.write_u1(flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::GtF(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b10))?;
                writer.write_u1(flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::LtF(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b11))?;
                writer.write_u1(flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }

            CmpOp::GtR(reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b00))?;
                writer.write_u1(u1::with(0b0))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::LtR(reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b00))?;
                writer.write_u1(u1::with(0b1))?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::EqA(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b01))?;
                writer.write_bool(*flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::EqF(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b10))?;
                writer.write_u1(flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }
            CmpOp::EqR(flag, reg, idx1, idx2) => {
                writer.write_u2(u2::with(0b11))?;
                writer.write_bool(*flag)?;
                writer.write_u5(idx1)?;
                writer.write_u5(idx2)?;
                writer.write_u3(reg)?;
            }

            CmpOp::IfZA(reg, idx) | CmpOp::IfNA(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            CmpOp::IfZR(reg, idx) | CmpOp::IfNR(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            CmpOp::St(flag, reg, idx) => {
                writer.write_u2(flag)?;
                writer.write_u3(reg)?;
                writer.write_u3(idx)?;
            }
            CmpOp::StInv => {}
        }
        Ok(())
    }

    fn read<R>(reader: &mut R) -> Result<Self, DecodeError>
    where
        R: Read,
        DecodeError: From<<R as Read>::Error>,
    {
        let instr = reader.read_u8()?;

        Ok(if instr == INSTR_LGT || instr == INSTR_CMP {
            let code = reader.read_u2()?;
            let flag = reader.read_u1()?;
            let idx1 = reader.read_u5()?.into();
            let idx2 = reader.read_u5()?.into();
            let reg = reader.read_u3()?;
            match (instr, code.as_u8(), flag.as_u8()) {
                (INSTR_LGT, 0b00, _) => CmpOp::GtA(flag.into(), reg.into(), idx1, idx2),
                (INSTR_LGT, 0b01, _) => CmpOp::LtA(flag.into(), reg.into(), idx1, idx2),
                (INSTR_LGT, 0b10, _) => CmpOp::GtF(flag.into(), reg.into(), idx1, idx2),
                (INSTR_LGT, 0b11, _) => CmpOp::LtF(flag.into(), reg.into(), idx1, idx2),
                (INSTR_CMP, 0b00, 0b0) => CmpOp::GtR(reg.into(), idx1, idx2),
                (INSTR_CMP, 0b00, 0b1) => CmpOp::LtR(reg.into(), idx1, idx2),
                (INSTR_CMP, 0b01, _) => CmpOp::EqA(flag.as_u8() == 1, reg.into(), idx1, idx2),
                (INSTR_CMP, 0b10, _) => CmpOp::EqF(flag.into(), reg.into(), idx1, idx2),
                (INSTR_CMP, 0b11, _) => CmpOp::EqR(flag.as_u8() == 1, reg.into(), idx1, idx2),
                _ => unreachable!(),
            }
        } else if instr == INSTR_STINV {
            CmpOp::StInv
        } else if instr == INSTR_ST {
            CmpOp::St(reader.read_u2()?.into(), reader.read_u3()?.into(), reader.read_u3()?.into())
        } else {
            let reg = reader.read_u3()?;
            let idx = reader.read_u5()?.into();
            match instr {
                INSTR_IFZA => CmpOp::IfZA(reg.into(), idx),
                INSTR_IFNA => CmpOp::IfNA(reg.into(), idx),
                INSTR_IFZR => CmpOp::IfZR(reg.into(), idx),
                INSTR_IFNR => CmpOp::IfNR(reg.into(), idx),
                x => unreachable!("instruction {:#010b} classified as comparison operation", x),
            }
        })
    }
}

impl Bytecode for ArithmeticOp {
    fn byte_count(&self) -> u16 {
        match self {
            ArithmeticOp::AddA(_, _, _, _)
            | ArithmeticOp::AddF(_, _, _, _)
            | ArithmeticOp::SubA(_, _, _, _)
            | ArithmeticOp::SubF(_, _, _, _)
            | ArithmeticOp::MulA(_, _, _, _)
            | ArithmeticOp::MulF(_, _, _, _)
            | ArithmeticOp::DivA(_, _, _, _)
            | ArithmeticOp::DivF(_, _, _, _)
            | ArithmeticOp::Rem(_, _, _, _) => 3,
            ArithmeticOp::Stp(_, _, _) => 4,
            ArithmeticOp::Neg(_, _) | ArithmeticOp::Abs(_, _) => 2,
        }
    }

    fn instr_range() -> RangeInclusive<u8> { INSTR_ADD..=INSTR_REM }

    fn instr_byte(&self) -> u8 {
        match self {
            ArithmeticOp::AddF(_, _, _, _) | ArithmeticOp::AddA(_, _, _, _) => INSTR_ADD,
            ArithmeticOp::SubF(_, _, _, _) | ArithmeticOp::SubA(_, _, _, _) => INSTR_SUB,
            ArithmeticOp::MulF(_, _, _, _) | ArithmeticOp::MulA(_, _, _, _) => INSTR_MUL,
            ArithmeticOp::DivF(_, _, _, _) | ArithmeticOp::DivA(_, _, _, _) => INSTR_DIV,
            ArithmeticOp::Rem(_, _, _, _) => INSTR_REM,
            ArithmeticOp::Stp(_, _, _) => INSTR_STP,
            ArithmeticOp::Neg(_, _) => INSTR_NEG,
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
                writer.write_u4(reg)?;
                writer.write_u4(idx)?;
            }
            ArithmeticOp::Stp(reg, idx, step) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
                writer.write_i16(step.as_i16())?;
            }
            ArithmeticOp::AddA(flags, reg, src1, src2)
            | ArithmeticOp::SubA(flags, reg, src1, src2)
            | ArithmeticOp::MulA(flags, reg, src1, src2)
            | ArithmeticOp::DivA(flags, reg, src1, src2) => {
                writer.write_u1(u1::with(0b0))?;
                writer.write_u2(flags)?;
                writer.write_u5(src1)?;
                writer.write_u5(src2)?;
                writer.write_u3(reg)?;
            }
            ArithmeticOp::AddF(flag, reg, src1, src2)
            | ArithmeticOp::SubF(flag, reg, src1, src2)
            | ArithmeticOp::MulF(flag, reg, src1, src2)
            | ArithmeticOp::DivF(flag, reg, src1, src2) => {
                writer.write_u1(u1::with(0b1))?;
                writer.write_u2(flag)?;
                writer.write_u5(src1)?;
                writer.write_u5(src2)?;
                writer.write_u3(reg)?;
            }
            ArithmeticOp::Rem(reg1, src1, reg2, src2) => {
                writer.write_u3(reg1)?;
                writer.write_u5(src1)?;
                writer.write_u3(reg2)?;
                writer.write_u5(src2)?;
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

        Ok(if instr >= INSTR_ADD && instr <= INSTR_DIV {
            let code = reader.read_u1()?.into();
            let flags = reader.read_u2()?;
            let src1 = reader.read_u5()?.into();
            let src2 = reader.read_u5()?.into();
            let reg = reader.read_u3()?;
            match (code, instr) {
                (0b0, INSTR_ADD) => Self::AddA(flags.into(), reg.into(), src1, src2),
                (0b0, INSTR_SUB) => Self::SubA(flags.into(), reg.into(), src1, src2),
                (0b0, INSTR_MUL) => Self::MulA(flags.into(), reg.into(), src1, src2),
                (0b0, INSTR_DIV) => Self::DivA(flags.into(), reg.into(), src1, src2),
                (0b1, INSTR_ADD) => Self::AddF(flags.into(), reg.into(), src1, src2),
                (0b1, INSTR_SUB) => Self::SubF(flags.into(), reg.into(), src1, src2),
                (0b1, INSTR_MUL) => Self::MulF(flags.into(), reg.into(), src1, src2),
                (0b1, INSTR_DIV) => Self::DivF(flags.into(), reg.into(), src1, src2),
                _ => unreachable!(),
            }
        } else {
            match instr {
                INSTR_NEG => Self::Neg(reader.read_u4()?.into(), reader.read_u4()?.into()),
                INSTR_STP => {
                    let reg = reader.read_u3()?.into();
                    let idx = reader.read_u5()?.into();
                    let step = reader.read_i16()?.into();
                    Self::Stp(reg, idx, step)
                }
                INSTR_REM => {
                    let reg1 = reader.read_u3()?.into();
                    let src1 = reader.read_u5()?.into();
                    let reg2 = reader.read_u3()?.into();
                    let src2 = reader.read_u5()?.into();
                    Self::Rem(reg1, src1, reg2, src2)
                }
                INSTR_ABS => Self::Abs(reader.read_u4()?.into(), reader.read_u4()?.into()),
                x => unreachable!("instruction {:#010b} classified as arithmetic operation", x),
            }
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
            | BitwiseOp::ShrA(_, _, _, _, _)
            | BitwiseOp::ShrR(_, _, _, _)
            | BitwiseOp::Scl(_, _, _, _)
            | BitwiseOp::Scr(_, _, _, _) => 3,

            BitwiseOp::RevA(_, _) | BitwiseOp::RevR(_, _) => 2,
        }
    }

    fn instr_range() -> RangeInclusive<u8> { INSTR_AND..=INSTR_REVR }

    fn instr_byte(&self) -> u8 {
        match self {
            BitwiseOp::And(_, _, _, _) => INSTR_AND,
            BitwiseOp::Or(_, _, _, _) => INSTR_OR,
            BitwiseOp::Xor(_, _, _, _) => INSTR_XOR,
            BitwiseOp::Not(_, _) => INSTR_NOT,

            BitwiseOp::Shl(_, _, _, _) => INSTR_SHF,
            BitwiseOp::ShrA(_, _, _, _, _) => INSTR_SHF,
            BitwiseOp::ShrR(_, _, _, _) => INSTR_SHF,
            BitwiseOp::Scl(_, _, _, _) => INSTR_SHC,
            BitwiseOp::Scr(_, _, _, _) => INSTR_SHC,

            BitwiseOp::RevA(_, _) => INSTR_REVA,
            BitwiseOp::RevR(_, _) => INSTR_REVR,
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
            | BitwiseOp::Xor(reg, idx1, idx2, idx3) => {
                writer.write_u4(reg)?;
                writer.write_u4(idx1)?;
                writer.write_u4(idx2)?;
                writer.write_u4(idx3)?;
            }
            BitwiseOp::Not(reg, idx) => {
                writer.write_u4(reg)?;
                writer.write_u4(idx)?;
            }

            BitwiseOp::Shl(a2, shift, reg, idx) => {
                writer.write_u1(u1::with(0b0))?;
                writer.write_u1(a2)?;
                writer.write_u5(shift)?;
                writer.write_u4(reg)?;
                writer.write_u5(idx)?;
            }
            BitwiseOp::ShrA(sign, a2, shift, reg, idx) => {
                writer.write_u1(u1::with(0b1))?;
                writer.write_u1(a2)?;
                writer.write_u4(shift)?;
                writer.write_u1(sign)?;
                writer.write_u1(u1::with(0b0))?;
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            BitwiseOp::ShrR(a2, shift, reg, idx) => {
                writer.write_u1(u1::with(0b1))?;
                writer.write_u1(a2)?;
                writer.write_u5(shift)?;
                writer.write_u1(u1::with(0b1))?;
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }

            BitwiseOp::Scl(a2, shift, reg, idx) => {
                writer.write_u1(u1::with(0b0))?;
                writer.write_u1(a2)?;
                writer.write_u5(shift)?;
                writer.write_u4(reg)?;
                writer.write_u5(idx)?;
            }
            BitwiseOp::Scr(a2, shift, reg, idx) => {
                writer.write_u1(u1::with(0b1))?;
                writer.write_u1(a2)?;
                writer.write_u5(shift)?;
                writer.write_u4(reg)?;
                writer.write_u5(idx)?;
            }

            BitwiseOp::RevA(reg, idx) => {
                writer.write_u3(reg)?;
                writer.write_u5(idx)?;
            }
            BitwiseOp::RevR(reg, idx) => {
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

        Ok(if instr >= INSTR_AND && instr <= INSTR_XOR {
            let reg = reader.read_u4()?.into();
            let src1 = reader.read_u4()?.into();
            let src2 = reader.read_u4()?.into();
            let dst = reader.read_u4()?.into();
            match instr {
                INSTR_AND => Self::And(reg, src1, src2, dst),
                INSTR_OR => Self::Or(reg, src1, src2, dst),
                INSTR_XOR => Self::Xor(reg, src1, src2, dst),
                _ => unreachable!(),
            }
        } else if instr == INSTR_SHC {
            let code = reader.read_u1()?;
            let a2 = reader.read_u1()?.into();
            let shift = reader.read_u5()?.into();
            let reg = reader.read_u4()?.into();
            let idx = reader.read_u5()?.into();
            match code.as_u8() {
                0b0 => Self::Scl(a2, shift, reg, idx),
                0b1 => Self::Scr(a2, shift, reg, idx),
                _ => unreachable!(),
            }
        } else if instr == INSTR_SHF {
            let code = reader.read_u1()?;
            let a2 = reader.read_u1()?.into();
            let shift = reader.read_u4()?;
            let sign = reader.read_u1()?;
            let block = reader.read_u1()?;
            let reg = reader.read_u3()?;
            let idx = reader.read_u5()?.into();
            let shift2 = u5::with(shift.as_u8() << 1 | sign.as_u8()).into();
            let regar = RegAR::from(block.into(), reg.into());
            match (code.as_u8(), block.as_u8()) {
                (0b0, _) => Self::Shl(a2, shift2, regar, idx),
                (0b1, 0b0) => Self::ShrA(sign.into(), a2, shift.into(), reg.into(), idx),
                (0b1, 0b1) => Self::ShrR(a2, shift2, reg.into(), idx),
                _ => unreachable!(),
            }
        } else {
            match instr {
                INSTR_NOT => Self::Not(reader.read_u4()?.into(), reader.read_u4()?.into()),
                INSTR_REVA => Self::RevA(reader.read_u3()?.into(), reader.read_u5()?.into()),
                INSTR_REVR => Self::RevR(reader.read_u3()?.into(), reader.read_u5()?.into()),
                x => unreachable!("instruction {:#010b} classified as bitwise operation", x),
            }
        })
    }
}

impl Bytecode for BytesOp {
    fn byte_count(&self) -> u16 {
        match self {
            BytesOp::Put(_, ByteStr { len, .. }) => 4u16.saturating_add(*len),
            BytesOp::Mov(_, _) | BytesOp::Swp(_, _) => 3,
            BytesOp::Fill(_, _, _, _, _) => 3,
            BytesOp::Len(_, _, _) | BytesOp::Cnt(_, _, _) | BytesOp::Eq(_, _) => 3,
            BytesOp::Con(_, _, _, _, _) => 5,
            BytesOp::Find(_, _) => 2,
            BytesOp::Extr(_, _, _, _) | BytesOp::Inj(_, _, _, _) => 3,
            BytesOp::Join(_, _, _) => 4,
            BytesOp::Splt(_, _, _, _, _) => 5,
            BytesOp::Ins(_, _, _, _) => 4,
            BytesOp::Del(_, _, _, _, _, _, _, _, _) => 5,
            BytesOp::Rev(_, _) => 3,
        }
    }

    fn instr_range() -> RangeInclusive<u8> { INSTR_PUT..=INSTR_REV }

    fn instr_byte(&self) -> u8 {
        match self {
            BytesOp::Put(_, _) => INSTR_PUT,
            BytesOp::Mov(_, _) => INSTR_MVS,
            BytesOp::Swp(_, _) => INSTR_SWP,
            BytesOp::Fill(_, _, _, _, _) => INSTR_FILL,
            BytesOp::Len(_, _, _) => INSTR_LEN,
            BytesOp::Cnt(_, _, _) => INSTR_CNT,
            BytesOp::Eq(_, _) => INSTR_EQ,
            BytesOp::Con(_, _, _, _, _) => INSTR_CON,
            BytesOp::Find(_, _) => INSTR_FIND,
            BytesOp::Extr(_, _, _, _) => INSTR_EXTR,
            BytesOp::Inj(_, _, _, _) => INSTR_INJ,
            BytesOp::Join(_, _, _) => INSTR_JOIN,
            BytesOp::Splt(_, _, _, _, _) => INSTR_SPLT,
            BytesOp::Ins(_, _, _, _) => INSTR_DEL,
            BytesOp::Del(_, _, _, _, _, _, _, _, _) => INSTR_INS,
            BytesOp::Rev(_, _) => INSTR_REV,
        }
    }

    fn write_args<W>(&self, writer: &mut W) -> Result<(), EncodeError>
    where
        W: Write,
        EncodeError: From<<W as Write>::Error>,
    {
        match self {
            BytesOp::Put(reg, bytes) => {
                writer.write_u8(*reg)?;
                writer.write_slice(bytes.as_ref())?;
            }
            BytesOp::Mov(reg1, reg2)
            | BytesOp::Swp(reg1, reg2)
            | BytesOp::Find(reg1, reg2)
            | BytesOp::Rev(reg1, reg2) => {
                writer.write_u8(*reg1)?;
                writer.write_u8(*reg2)?;
            }
            BytesOp::Fill(reg, offset1, offset2, value, flag) => {
                writer.write_u8(*reg)?;
                writer.write_u5(offset1)?;
                writer.write_u5(offset2)?;
                writer.write_u5(value)?;
                writer.write_bool(*flag)?;
            }
            BytesOp::Len(src, reg, dst) => {
                writer.write_u8(*src)?;
                writer.write_u3(reg)?;
                writer.write_u5(dst)?;
            }
            BytesOp::Cnt(src, byte, cnt) => {
                writer.write_u8(*src)?;
                writer.write_u4(byte)?;
                writer.write_u4(cnt)?;
            }
            BytesOp::Eq(reg1, reg2) => {
                writer.write_u8(*reg1)?;
                writer.write_u8(*reg2)?;
            }
            BytesOp::Con(reg1, reg2, no, offset, len) => {
                writer.write_u8(*reg1)?;
                writer.write_u8(*reg2)?;
                writer.write_u6(*no)?;
                writer.write_u5(offset)?;
                writer.write_u5(len)?;
            }
            BytesOp::Extr(src, dst, index, offset) | BytesOp::Inj(src, dst, index, offset) => {
                writer.write_u5(src)?;
                writer.write_u3(dst)?;
                writer.write_u4(index)?;
                writer.write_u4(offset)?;
            }
            BytesOp::Join(src1, src2, dst) => {
                writer.write_u8(*src1)?;
                writer.write_u8(*src2)?;
                writer.write_u8(*dst)?;
            }
            BytesOp::Splt(flag, offset, src, dst1, dst2) => {
                writer.write_u3(flag)?;
                writer.write_u5(offset)?;
                writer.write_u8(*src)?;
                writer.write_u8(*dst1)?;
                writer.write_u8(*dst2)?;
            }
            BytesOp::Ins(flag, offset, src, dst) => {
                writer.write_u3(flag)?;
                writer.write_u5(offset)?;
                writer.write_u8(*src)?;
                writer.write_u8(*dst)?;
            }
            BytesOp::Del(flag, reg1, offset1, reg2, offset2, flag1, flag2, src, dst) => {
                writer.write_u2(flag)?;
                writer.write_u1(reg1)?;
                writer.write_u5(offset1)?;
                writer.write_u1(reg2)?;
                writer.write_u5(offset2)?;
                writer.write_bool(*flag1)?;
                writer.write_bool(*flag2)?;
                writer.write_u8(*src)?;
                writer.write_u8(*dst)?;
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
            INSTR_PUT => Self::Put(reader.read_u8()?, ByteStr::with(reader.read_slice()?)),
            INSTR_MVS => Self::Mov(reader.read_u8()?, reader.read_u8()?),
            INSTR_SWP => Self::Swp(reader.read_u8()?, reader.read_u8()?),
            INSTR_FIND => Self::Find(reader.read_u8()?, reader.read_u8()?),
            INSTR_REV => Self::Rev(reader.read_u8()?, reader.read_u8()?),

            INSTR_FILL => Self::Fill(
                reader.read_u8()?,
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_bool()?,
            ),
            INSTR_LEN => {
                Self::Len(reader.read_u8()?, reader.read_u3()?.into(), reader.read_u5()?.into())
            }
            INSTR_CNT => {
                Self::Cnt(reader.read_u8()?, reader.read_u4()?.into(), reader.read_u4()?.into())
            }
            INSTR_EQ => Self::Eq(reader.read_u8()?, reader.read_u8()?),
            INSTR_CON => Self::Con(
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u6()?,
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_EXTR => Self::Extr(
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u4()?.into(),
                reader.read_u4()?.into(),
            ),
            INSTR_INJ => Self::Inj(
                reader.read_u5()?.into(),
                reader.read_u3()?.into(),
                reader.read_u4()?.into(),
                reader.read_u4()?.into(),
            ),
            INSTR_JOIN => Self::Join(reader.read_u8()?, reader.read_u8()?, reader.read_u8()?),
            INSTR_SPLT => Self::Splt(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u8()?,
            ),
            INSTR_INS => Self::Ins(
                reader.read_u3()?.into(),
                reader.read_u5()?.into(),
                reader.read_u8()?,
                reader.read_u8()?,
            ),
            INSTR_DEL => Self::Del(
                reader.read_u2()?.into(),
                reader.read_u1()?.into(),
                reader.read_u5()?.into(),
                reader.read_u1()?.into(),
                reader.read_u5()?.into(),
                reader.read_bool()?,
                reader.read_bool()?,
                reader.read_u8()?,
                reader.read_u8()?,
            ),
            x => unreachable!("instruction {:#010b} classified as byte string operation", x),
        })
    }
}

impl Bytecode for DigestOp {
    fn byte_count(&self) -> u16 { 3 }

    fn instr_range() -> RangeInclusive<u8> { INSTR_RIPEMD..=INSTR_HASH5 }

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

    fn instr_range() -> RangeInclusive<u8> { INSTR_SECP_GEN..=INSTR_SECP_NEG }

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
                writer.write_bool(*reg == RegBlockAR::A)?;
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
                if reader.read_bool()? { RegBlockAR::A } else { RegBlockAR::R },
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
                reader.read_u5()?.into(),
            ),
            INSTR_SECP_ADD => Self::Add(reader.read_u5()?.into(), reader.read_u3()?.into()),
            INSTR_SECP_NEG => Self::Neg(reader.read_u5()?.into(), reader.read_u3()?.into()),
            x => unreachable!("instruction {:#010b} classified as Secp256k1 curve operation", x),
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

    fn instr_range() -> RangeInclusive<u8> { INSTR_ED_GEN..=INSTR_ED_NEG }

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
                writer.write_bool(*reg == RegBlockAR::A)?;
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
                if reader.read_bool()? { RegBlockAR::A } else { RegBlockAR::R },
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
    fn byte_count(&self) -> u16 { 1 }

    fn instr_range() -> RangeInclusive<u8> { INSTR_NOP..=INSTR_NOP }

    fn instr_byte(&self) -> u8 { INSTR_NOP }

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
