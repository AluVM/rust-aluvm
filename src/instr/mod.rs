// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_use]
mod asm;
#[cfg(feature = "std")]
pub mod encoding;
mod instr;

pub use encoding::Bytecode;
pub use instr::*;

use amplify::num::{u1024, u2, u3, u4, u512};
#[cfg(feature = "std")]
use std::fmt::{self, Display, Formatter};

use crate::registers::{Reg, Reg32, Reg8, RegA, RegBlock, RegR, Registers};
use crate::types::Blob;
use crate::{LibSite, Value};

/// Turing machine movement after instruction execution
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ExecStep {
    /// Stop program execution
    Stop,

    /// Move to the next instruction
    Next,

    /// Jump to the offset from the origin
    Jump(u16),

    /// Jump to another code fragment
    Call(LibSite),
}

#[cfg(not(feature = "std"))]
/// Trait for instructions
pub trait Instruction: Bytecode {
    /// Executes given instruction taking all registers as input and output.
    /// The method is provided with the current code position which may be
    /// used by the instruction for constructing call stack.
    ///
    /// Returns whether further execution should be stopped.
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep;
}

#[cfg(feature = "std")]
/// Trait for instructions
pub trait InstructionSet: Bytecode + Display {
    /// Executes given instruction taking all registers as input and output.
    /// The method is provided with the current code position which may be
    /// used by the instruction for constructing call stack.
    ///
    /// Returns whether further execution should be stopped.
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep;
}

/// Default instruction extension which treats any operation as NOP
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display), display("nop"))]
pub enum Nop {}
impl InstructionSet for Nop {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        ExecStep::Next
    }
}

/// Full set of instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display), display(inner))]
#[non_exhaustive]
pub enum Instr<Extension = Nop>
where
    Extension: InstructionSet,
{
    /// Control-flow instructions
    // 0b00_000_***
    ControlFlow(ControlFlowOp),

    /// Instructions setting register values
    // 0b00_001_***
    Put(PutOp),

    /// Instructions moving and swapping register values
    // 0b00_010_***
    Move(MoveOp),

    /// Instructions comparing register values
    // 0b00_011_***
    Cmp(CmpOp),

    /// Arithmetic instructions
    // 0b00_100_***
    Arithmetic(ArithmeticOp),

    /// Bit operations & boolean algebra instructions
    // 0b00_101_***
    Bitwise(BitwiseOp),

    /// Operations on byte strings
    // 0b00_110_***
    Bytes(BytesOp),

    /// Cryptographic hashing functions
    // 0b01_000_***
    Digest(DigestOp),

    /// Operations on Secp256k1 elliptic curve
    // 0b01_001_0**
    Secp256k1(SecpOp),

    /// Operations on Curve25519 elliptic curve
    // 0b01_001_1**
    Curve25519(Curve25519Op),

    /// Extension operations which can be provided by a host environment
    // 0b10_***_***
    ExtensionCodes(Extension),

    // Reserved operations for future use.
    //
    // When such an opcode is met in the bytecode the decoder MUST fail.
    // 0x11_***_***
    /// No-operation instruction
    // #[value = 0b11_111_111]
    Nop,
}

impl<Extension> InstructionSet for Instr<Extension>
where
    Extension: InstructionSet,
{
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        match self {
            Instr::ControlFlow(instr) => instr.exec(regs, site),
            Instr::Put(instr) => instr.exec(regs, site),
            Instr::Move(instr) => instr.exec(regs, site),
            Instr::Cmp(instr) => instr.exec(regs, site),
            Instr::Arithmetic(instr) => instr.exec(regs, site),
            Instr::Bitwise(instr) => instr.exec(regs, site),
            Instr::Bytes(instr) => instr.exec(regs, site),
            Instr::Digest(instr) => instr.exec(regs, site),
            Instr::Secp256k1(instr) => instr.exec(regs, site),
            Instr::Curve25519(instr) => instr.exec(regs, site),
            Instr::ExtensionCodes(instr) => instr.exec(regs, site),
            Instr::Nop => ExecStep::Next,
        }
    }
}

/// Control-flow instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum ControlFlowOp {
    /// Completes program execution writing `false` to `st0` (indicating
    /// program failure)
    #[cfg_attr(feature = "std", display("fail"))]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program
    /// success)
    #[cfg_attr(feature = "std", display("succ"))]
    Succ,

    /// Unconditionally jumps to an offset. Increments `cy0`.
    #[cfg_attr(feature = "std", display("jmp\t\t{0:#06X}"))]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments
    /// `cy0`.
    #[cfg_attr(feature = "std", display("jif\t\t{0:#06X}"))]
    Jif(u16),

    /// Jumps to other location in the current code with ability to return
    /// back (calls a subroutine). Increments `cy0` and pushes offset of the
    /// instruction which follows current one to `cs0`.
    #[cfg_attr(feature = "std", display("routine\t{0:#06X}"))]
    Routine(u16),

    /// Calls code from an external library identified by the hash of its code.
    /// Increments `cy0` and `cp0` and pushes offset of the instruction which
    /// follows current one to `cs0`.
    #[cfg_attr(feature = "std", display("call\t{0}"))]
    Call(LibSite),

    /// Passes execution to other library without an option to return.
    /// Does not increments `cy0` and `cp0` counters and does not add anything
    /// to the call stack `cs0`.
    #[cfg_attr(feature = "std", display("exec\t{0}"))]
    Exec(LibSite),

    /// Returns execution flow to the previous location from the top of `cs0`.
    /// Does not change value in `cy0`. Decrements `cp0`.
    #[cfg_attr(feature = "std", display("ret"))]
    Ret,
}

impl InstructionSet for ControlFlowOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        match self {
            ControlFlowOp::Fail => {
                regs.st0 = false;
                ExecStep::Stop
            }
            ControlFlowOp::Succ => {
                regs.st0 = true;
                ExecStep::Stop
            }
            ControlFlowOp::Jmp(offset) => regs
                .jmp()
                .map(|_| ExecStep::Jump(offset))
                .unwrap_or(ExecStep::Stop),
            ControlFlowOp::Jif(offset) => {
                if regs.st0 == true {
                    regs.jmp()
                        .map(|_| ExecStep::Jump(offset))
                        .unwrap_or(ExecStep::Stop)
                } else {
                    ExecStep::Next
                }
            }
            ControlFlowOp::Routine(offset) => regs
                .call(site)
                .map(|_| ExecStep::Jump(offset))
                .unwrap_or(ExecStep::Stop),
            ControlFlowOp::Call(site) => regs
                .call(site)
                .map(|_| ExecStep::Call(site))
                .unwrap_or(ExecStep::Stop),
            ControlFlowOp::Exec(site) => regs
                .jmp()
                .map(|_| ExecStep::Call(site))
                .unwrap_or(ExecStep::Stop),
            ControlFlowOp::Ret => {
                regs.ret().map(ExecStep::Call).unwrap_or(ExecStep::Stop)
            }
        }
    }
}

/// Instructions setting register values
#[cfg_attr(feature = "std", derive(Display))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PutOp {
    /// Sets `a` register value to zero
    #[cfg_attr(feature = "std", display("zero\t{0}{1}"))]
    ZeroA(RegA, Reg32),

    /// Sets `r` register value to zero
    #[cfg_attr(feature = "std", display("zero\t{0}{1}"))]
    ZeroR(RegR, Reg32),

    /// Cleans a value of `a` register (sets it to undefined state)
    #[cfg_attr(feature = "std", display("cl\t\t{0}{1}"))]
    ClA(RegA, Reg32),

    /// Cleans a value of `r` register (sets it to undefined state)
    #[cfg_attr(feature = "std", display("cl\t\t{0}{1}"))]
    ClR(RegR, Reg32),

    /// Unconditionally assigns a value to `a` register
    #[cfg_attr(feature = "std", display("put\t\t{0}{1}, {2}"))]
    PutA(RegA, Reg32, Value),

    /// Unconditionally assigns a value to `r` register
    #[cfg_attr(feature = "std", display("put\t\t{0}{1}, {2}"))]
    PutR(RegR, Reg32, Value),

    /// Conditionally assigns a value to `a` register if the register is in
    /// uninitialized state
    #[cfg_attr(feature = "std", display("putif\t{0}{1}, {2}"))]
    PutIfA(RegA, Reg32, Value),

    /// Conditionally assigns a value to `r` register if the register is in
    /// uninitialized state
    #[cfg_attr(feature = "std", display("putif\t{0}{1}, {2}"))]
    PutIfR(RegR, Reg32, Value),
}

impl InstructionSet for PutOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            PutOp::ZeroA(reg, index) => {
                regs.set(Reg::A(reg), index, Some(0.into()))
            }
            PutOp::ZeroR(reg, index) => {
                regs.set(Reg::R(reg), index, Some(0.into()))
            }
            PutOp::ClA(reg, index) => regs.set(Reg::A(reg), index, None),
            PutOp::ClR(reg, index) => regs.set(Reg::R(reg), index, None),
            PutOp::PutA(reg, index, blob) => {
                regs.set(Reg::A(reg), index, Some(blob))
            }
            PutOp::PutR(reg, index, blob) => {
                regs.set(Reg::R(reg), index, Some(blob))
            }
            PutOp::PutIfA(reg, index, blob) => {
                regs.get(Reg::A(reg), index).or_else(|| {
                    regs.set(Reg::A(reg), index, Some(blob));
                    Some(blob)
                });
            }
            PutOp::PutIfR(reg, index, blob) => {
                regs.get(Reg::R(reg), index).or_else(|| {
                    regs.set(Reg::R(reg), index, Some(blob));
                    Some(blob)
                });
            }
        }
        ExecStep::Next
    }
}

/// Integer arithmetic types
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum NumType {
    /// Unsigned integer
    #[cfg_attr(feature = "std", display("u"))]
    Unsigned,

    /// Signed integer
    #[cfg_attr(feature = "std", display("s"))]
    Signed,

    /// Float number with 23-bit mantissa
    #[cfg_attr(feature = "std", display("f"))]
    Float23,

    /// Float number with 52 bit mantissa
    #[cfg_attr(feature = "std", display("d"))]
    Float52,
}

impl NumType {
    /// Constructs numeric type from `u2` value (used in bytecode serialization)
    pub fn from_u2(val: u2) -> NumType {
        match *val {
            0 => NumType::Unsigned,
            1 => NumType::Signed,
            2 => NumType::Float23,
            3 => NumType::Float52,
            _ => unreachable!(),
        }
    }

    /// Returns `u2` representation of numeric type (used in bytecode
    /// serialization).
    pub fn as_u2(self) -> u2 {
        match self {
            NumType::Unsigned => u2::with(0),
            NumType::Signed => u2::with(1),
            NumType::Float23 => u2::with(2),
            NumType::Float52 => u2::with(3),
        }
    }
}

impl From<u2> for NumType {
    fn from(val: u2) -> NumType {
        NumType::from_u2(val)
    }
}

impl From<&NumType> for u2 {
    fn from(nt: &NumType) -> u2 {
        nt.as_u2()
    }
}

impl From<NumType> for u2 {
    fn from(nt: NumType) -> u2 {
        nt.as_u2()
    }
}

/// Instructions moving and swapping register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum MoveOp {
    /// Swap operation for arithmetic registers. If the value does not fit
    /// destination bit dimensions truncates the most significant bits until
    /// they fit.
    #[cfg_attr(feature = "std", display("swp\t\t{0}{1},{2}{3}"))]
    SwpA(RegA, Reg32, RegA, Reg32),

    /// Swap operation for non-arithmetic registers. If the value does not fit
    /// destination bit dimensions truncates the most significant bits until
    /// they fit.
    #[cfg_attr(feature = "std", display("swp\t\t{0}{1},{2}{3}"))]
    SwpR(RegR, Reg32, RegR, Reg32),

    /// Swap operation between arithmetic and non-arithmetic registers. If the
    /// value does not fit destination bit dimensions truncates the most
    /// significant bits until they fit.
    #[cfg_attr(feature = "std", display("swp\t\t{0}{1},{2}{3}"))]
    SwpAR(RegA, Reg32, RegR, Reg32),

    /// Array move operation: duplicates values of all register set into
    /// another set
    #[cfg_attr(feature = "std", display("amov:{2}\t{0},{1}"))]
    AMov(RegA, RegA, NumType),

    /// Move operation: duplicates value of one of the arithmetic registers
    /// into another arithmetic register
    #[cfg_attr(feature = "std", display("mov\t\t{0}{1},{2}{3}"))]
    MovA(RegA, Reg32, RegA, Reg32),

    /// Move operation: duplicates value of one of the non-arithmetic registers
    /// into another non-arithmetic register
    #[cfg_attr(feature = "std", display("mov\t\t{0}{1},{2}{3}"))]
    MovR(RegR, Reg32, RegR, Reg32),

    /// Move operation: duplicates value of one of the arithmetic registers
    /// into non-arithmetic register
    #[cfg_attr(feature = "std", display("mov\t\t{0}{1},{2}{3}"))]
    MovAR(RegA, Reg32, RegR, Reg32),

    /// Move operation: duplicates value of one of the n on-arithmetic
    /// registers into arithmetic register
    #[cfg_attr(feature = "std", display("mov\t\t{0}{1},{2}{3}"))]
    MovRA(RegR, Reg32, RegA, Reg32),
}

impl InstructionSet for MoveOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            MoveOp::SwpA(reg1, index1, reg2, index2) => {
                let val1 = regs.get(Reg::A(reg1), index1);
                let val2 = regs.get(Reg::A(reg2), index2);
                regs.set(Reg::A(reg1), index1, val2);
                regs.set(Reg::A(reg2), index2, val1);
            }
            MoveOp::SwpR(reg1, index1, reg2, index2) => {
                let val1 = regs.get(Reg::R(reg1), index1);
                let val2 = regs.get(Reg::R(reg2), index2);
                regs.set(Reg::R(reg1), index1, val2);
                regs.set(Reg::R(reg2), index2, val1);
            }
            MoveOp::SwpAR(reg1, index1, reg2, index2) => {
                let val1 = regs.get(Reg::A(reg1), index1);
                let val2 = regs.get(Reg::R(reg2), index2);
                regs.set(Reg::A(reg1), index1, val2);
                regs.set(Reg::R(reg2), index2, val1);
            }
            MoveOp::AMov(reg1, reg2, ty) => {
                match ty {
                    NumType::Unsigned => {}
                    NumType::Signed => {}
                    NumType::Float23 => {}
                    NumType::Float52 => {}
                }
                // TODO: array move operation
            }
            MoveOp::MovA(sreg, sidx, dreg, didx) => {
                regs.set(Reg::A(dreg), didx, regs.get(Reg::A(sreg), sidx));
            }
            MoveOp::MovR(sreg, sidx, dreg, didx) => {
                regs.set(Reg::R(dreg), didx, regs.get(Reg::R(sreg), sidx));
            }
            MoveOp::MovAR(sreg, sidx, dreg, didx) => {
                regs.set(Reg::R(dreg), didx, regs.get(Reg::A(sreg), sidx));
            }
            MoveOp::MovRA(sreg, sidx, dreg, didx) => {
                regs.set(Reg::A(dreg), didx, regs.get(Reg::R(sreg), sidx));
            }
        }
        ExecStep::Next
    }
}

/// Instructions comparing register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum CmpOp {
    /// Compares value of two arithmetic (`A`) registers setting `st0` to
    /// `true` if the first parameter is greater (and not equal) than the
    /// second one
    // #[value = 0b110] // 3 + 5 + 3 + 5 => 16 bits
    #[cfg_attr(feature = "std", display("gt\t\t{0}{1},{2}{3}"))]
    Gt(RegA, Reg32, RegA, Reg32),

    /// Compares value of two non-arithmetic (`R`) registers setting `st0` to
    /// `true` if the first parameter is less (and not equal) than the second
    /// one
    // #[value = 0b111]
    #[cfg_attr(feature = "std", display("lt\t\t{0}{1},{2}{3}"))]
    Lt(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two arithmetic (`A`) registers putting
    /// result into `st0`
    // #[value = 0b100]
    #[cfg_attr(feature = "std", display("eq\t\t{0}{1},{2}{3}"))]
    EqA(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting
    /// result into `st0`
    // #[value = 0b101]
    #[cfg_attr(feature = "std", display("eq\t\t{0}{1},{2}{3}"))]
    EqR(RegR, Reg32, RegR, Reg32),

    /// Measures bit length of a value in one fo the registers putting result
    /// to `a16[0]`
    #[cfg_attr(feature = "std", display("len\t\t{0}{1}"))]
    Len(RegA, Reg32),

    /// Counts number of `1` bits in register putting result to `a16[0]`
    /// register.
    #[cfg_attr(feature = "std", display("cnt\t\t{0}{1}"))]
    Cnt(RegA, Reg32),

    /// Assigns value of `a8[0]` register to `st0`
    #[cfg_attr(feature = "std", display("st2a"))]
    St2A,

    /// `st0` value of `st0` register to the result of `a8[0] == 1`
    #[cfg_attr(feature = "std", display("a2st"))]
    A2St,
}

impl InstructionSet for CmpOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        // TODO: Implement comparison operations
        ExecStep::Next
    }
}

/// Variants of arithmetic operations
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Arithmetics {
    IntChecked {
        /// Indicates the need to use signed integer arithmetics
        signed: bool,
    },
    IntUnchecked {
        /// Indicates the need to use signed integer arithmetics
        signed: bool,
    },
    IntArbitraryPrecision {
        /// Indicates the need to use signed integer arithmetics
        signed: bool,
    },
    Float,
    FloatArbitraryPrecision,
}

impl Arithmetics {
    /// Constructs arithmetics variant from `u3` value (used in bytecode
    /// serialization).
    pub fn from_u3(val: u3) -> Arithmetics {
        match *val {
            0 => Arithmetics::IntChecked { signed: false },
            1 => Arithmetics::IntUnchecked { signed: false },
            2 => Arithmetics::IntArbitraryPrecision { signed: false },
            3 => Arithmetics::IntChecked { signed: true },
            4 => Arithmetics::IntUnchecked { signed: true },
            5 => Arithmetics::IntArbitraryPrecision { signed: true },
            6 => Arithmetics::Float,
            7 => Arithmetics::FloatArbitraryPrecision,
            _ => unreachable!(),
        }
    }

    /// Returns `u3` representation of arithmetics variant (used in bytecode
    /// serialization).
    pub fn as_u3(self) -> u3 {
        match self {
            Arithmetics::IntChecked { signed: false } => u3::with(0),
            Arithmetics::IntUnchecked { signed: false } => u3::with(1),
            Arithmetics::IntArbitraryPrecision { signed: false } => u3::with(2),
            Arithmetics::IntChecked { signed: true } => u3::with(3),
            Arithmetics::IntUnchecked { signed: true } => u3::with(4),
            Arithmetics::IntArbitraryPrecision { signed: true } => u3::with(5),
            Arithmetics::Float => u3::with(6),
            Arithmetics::FloatArbitraryPrecision => u3::with(7),
        }
    }
}

impl From<u3> for Arithmetics {
    fn from(val: u3) -> Arithmetics {
        Arithmetics::from_u3(val)
    }
}

impl From<&Arithmetics> for u3 {
    fn from(ar: &Arithmetics) -> u3 {
        ar.as_u3()
    }
}

impl From<Arithmetics> for u3 {
    fn from(ar: Arithmetics) -> u3 {
        ar.as_u3()
    }
}

#[cfg(feature = "std")]
impl Display for Arithmetics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Arithmetics::IntChecked { signed: false } => f.write_str("c"),
            Arithmetics::IntUnchecked { signed: false } => f.write_str("u"),
            Arithmetics::IntArbitraryPrecision { signed: false } => {
                f.write_str("a")
            }
            Arithmetics::IntChecked { signed: true } => f.write_str("cs"),
            Arithmetics::IntUnchecked { signed: true } => f.write_str("us"),
            Arithmetics::IntArbitraryPrecision { signed: true } => {
                f.write_str("as")
            }
            Arithmetics::Float => f.write_str("f"),
            Arithmetics::FloatArbitraryPrecision => f.write_str("af"),
        }
    }
}

/// Selector between increment and decrement operation
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum IncDec {
    /// Increment operation
    #[display("inc")]
    Inc,

    /// Decrement operation
    #[display("dec")]
    Dec,
}

impl From<bool> for IncDec {
    fn from(val: bool) -> IncDec {
        if val {
            IncDec::Dec
        } else {
            IncDec::Inc
        }
    }
}

impl From<&IncDec> for bool {
    fn from(val: &IncDec) -> bool {
        bool::from(*val)
    }
}

impl From<IncDec> for bool {
    fn from(val: IncDec) -> bool {
        val == IncDec::Dec
    }
}

/// Arithmetic instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum ArithmeticOp {
    /// Negates most significant bit
    #[cfg_attr(feature = "std", display("neg\t\t{0}{1}"))]
    Neg(RegA, Reg32),

    /// Increases register value on a given step.
    #[cfg_attr(feature = "std", display("{0}:{1}\t{2}{3},{4}"))]
    Stp(IncDec, Arithmetics, RegA, Reg32, u4),

    /// Adds two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[cfg_attr(feature = "std", display("add:{0}\t{1}{2},{1}{3}"))]
    Add(Arithmetics, RegA, Reg32, Reg32),

    /// Subtracts two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[cfg_attr(feature = "std", display("sub:{0}\t{1}{2},{1}{3}"))]
    Sub(Arithmetics, RegA, Reg32, Reg32),

    /// Multiplies two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[cfg_attr(feature = "std", display("mul:{0}\t{1}{2},{1}{3}"))]
    Mul(Arithmetics, RegA, Reg32, Reg32),

    /// Divides two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[cfg_attr(feature = "std", display("div:{0}\t{1}{2},{1}{3}"))]
    Div(Arithmetics, RegA, Reg32, Reg32),

    /// Modulo division
    #[cfg_attr(feature = "std", display("mod\t\t{0}{1},{2}{3},{4}{5}"))]
    Mod(RegA, Reg32, RegA, Reg32, RegA, Reg32),

    /// Puts absolute value of register into `a8[0]`
    #[cfg_attr(feature = "std", display("abs\t\t{0}{1}"))]
    Abs(RegA, Reg32),
}

impl InstructionSet for ArithmeticOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            ArithmeticOp::Neg(reg, index) => {
                regs.get(Reg::A(reg), index).map(|mut blob| {
                    blob.bytes[reg as usize] = 0xFF ^ blob.bytes[reg as usize];
                    regs.set(Reg::A(reg), index, Some(blob));
                });
            }
            ArithmeticOp::Stp(dir, arithm, reg, index, step) => {
                regs.get(Reg::A(reg), index).map(|value| {
                    let u512_max = u512::from_le_bytes([0xFF; 64]);
                    let res = match arithm {
                        Arithmetics::IntChecked { signed: false } => {
                            let step = u512::from_u64(*step as u64).unwrap();
                            let mut val: u512 = value.into();
                            if step >= u512_max - val {
                                None
                            } else {
                                val = val + step;
                                Some(Value::from(val))
                            }
                        }
                        Arithmetics::IntUnchecked { signed: false } => {
                            let step = u512::from_u64(*step as u64).unwrap();
                            let mut val: u512 = value.into();
                            if step >= u512_max - val {
                                Some(Value::from(step - (u512_max - val)))
                            } else {
                                val = val + step;
                                Some(Value::from(val))
                            }
                        }
                        Arithmetics::IntArbitraryPrecision {
                            signed: false,
                        } => {
                            todo!("Arbitrary precision increment")
                        }
                        Arithmetics::IntChecked { signed: true } => {
                            todo!("Signed increment")
                        }
                        Arithmetics::IntUnchecked { signed: true } => {
                            todo!("Signed increment")
                        }
                        Arithmetics::IntArbitraryPrecision { signed: true } => {
                            todo!("Arbitrary precision signed increment")
                        }
                        Arithmetics::Float => todo!("Float increment"),
                        Arithmetics::FloatArbitraryPrecision => {
                            todo!("Float increment")
                        }
                    };
                    regs.set(Reg::A(reg), index, res);
                });
            }
            ArithmeticOp::Add(arithm, reg, src, dst) => {
                regs.get(Reg::A(reg), src).and_then(|value1| {
                    regs.get(Reg::A(reg), dst).map(|value2| (value1, value2))
                }).map(|(value1, value2)| {
                    let mut dst_reg = Reg::A(reg);
                    let res = match arithm {
                        Arithmetics::IntChecked { signed: false } => {
                            // TODO: Support source arbitrary precision registers
                            let mut val: u1024 = value1.into();
                            val = val + u1024::from(value2);
                            Value::from(val)
                        }
                        Arithmetics::IntUnchecked { signed: false } => {
                            // TODO: Support source arbitrary precision registers
                            let mut val: u1024 = value1.into();
                            val = val + u1024::from(value2);
                            Value::from(val)
                        }
                        Arithmetics::IntArbitraryPrecision {
                            signed: false,
                        } => {
                            dst_reg = Reg::A(RegA::AP);
                            todo!("Unsigned int addition with arbitrary precision")
                        }
                        Arithmetics::IntChecked { signed: true } => todo!("Signed int addition"),
                        Arithmetics::IntUnchecked { signed: true } => todo!("Signed int addition"),
                        Arithmetics::IntArbitraryPrecision { signed: true } => {
                            dst_reg = Reg::A(RegA::AP);
                            todo!("Signed int addition with arbitrary precision")
                        }
                        Arithmetics::Float => todo!("Float addition"),
                        Arithmetics::FloatArbitraryPrecision => {
                            dst_reg = Reg::A(RegA::AP);
                            todo!("Float addition with arbitrary precision")
                        }
                    };
                    regs.set(dst_reg, Reg32::Reg1, Some(res));
                });
            }
            ArithmeticOp::Sub(arithm, reg, src, dst) => {}
            ArithmeticOp::Mul(arithm, reg, src, dst) => {
                regs.get(Reg::A(reg), src).and_then(|value1| {
                    regs.get(Reg::A(reg), dst).map(|value2| (value1, value2))
                }).map(|(value1, value2)| {
                    let mut dst_reg = Reg::A(reg);
                    let res = match arithm {
                        Arithmetics::IntChecked { signed: false } => {
                            // TODO: Rewrite
                            let mut val: u1024 = value1.into();
                            val = val * u1024::from(value2);
                            Value::from(val)
                        }
                        Arithmetics::IntUnchecked { signed: false } => {
                            // TODO: Rewrite
                            let mut val: u1024 = value1.into();
                            val = val * u1024::from(value2);
                            Value::from(val)
                        }
                        Arithmetics::IntArbitraryPrecision {
                            signed: false,
                        } => {
                            dst_reg = Reg::A(RegA::AP);
                            todo!("Unsigned int multiplication with arbitrary precision")
                        }
                        Arithmetics::IntChecked { signed: true } => todo!("Signed int multiplication"),
                        Arithmetics::IntUnchecked { signed: true } => todo!("Signed int multiplication"),
                        Arithmetics::IntArbitraryPrecision { signed: true } => {
                            dst_reg = Reg::A(RegA::AP);
                            todo!("Signed int multiplication with arbitrary precision")
                        }
                        Arithmetics::Float => todo!("Float addition"),
                        Arithmetics::FloatArbitraryPrecision => {
                            dst_reg = Reg::A(RegA::AP);
                            todo!("Float multiplication with arbitrary precision")
                        }
                    };
                    regs.set(dst_reg, Reg32::Reg1, Some(res));
                });
            }
            ArithmeticOp::Div(arithm, reg, src, dst) => {}
            ArithmeticOp::Mod(reg1, index1, reg2, index2, reg3, index3) => {}
            ArithmeticOp::Abs(reg, index) => {}
        }
        ExecStep::Next
    }
}

/// Bit operations & boolean algebra instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum BitwiseOp {
    /// Bitwise AND operation
    #[cfg_attr(feature = "std", display("and\t\t{0}{1},{0}{2},{0}{3}"))]
    And(
        RegA,
        Reg32,
        Reg32,
        /// Operation destination, only first 8 registers
        Reg8,
    ),

    /// Bitwise OR operation
    #[cfg_attr(feature = "std", display("or\t\t{0}{1},{0}{2},{0}{3}"))]
    Or(RegA, Reg32, Reg32, Reg8),

    /// Bitwise XOR operation
    #[cfg_attr(feature = "std", display("xor\t\t{0}{1},{0}{2},{0}{3}"))]
    Xor(RegA, Reg32, Reg32, Reg8),

    /// Bitwise inversion
    #[cfg_attr(feature = "std", display("not\t\t{0}{1}"))]
    Not(RegA, Reg32),

    /// Left bit shift, filling added bits values with zeros
    #[cfg_attr(feature = "std", display("shl\t\t{0}{1},a8{2},{0}{3}"))]
    Shl(RegA, Reg32, Reg32 /* Always `a8` */, Reg8),

    /// Right bit shift, filling added bits values with zeros
    #[cfg_attr(feature = "std", display("shr\t\t{0}{1},a8{2},{0}{3}"))]
    Shr(RegA, Reg32, Reg32, Reg8),

    /// Left bit shift, cycling the shifted values (most significant bit
    /// becomes least significant)
    #[cfg_attr(feature = "std", display("scl\t\t{0}{1},a8{2},{0}{3}"))]
    Scl(RegA, Reg32, Reg32, Reg8),

    /// Right bit shift, cycling the shifted values (least significant bit
    /// becomes nost significant)
    #[cfg_attr(feature = "std", display("scr\t\t{0}{1},a8{2},{0}{3}"))]
    Scr(RegA, Reg32, Reg32, Reg8),
}

impl InstructionSet for BitwiseOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

/// Operations on byte strings
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum BytesOp {
    /// Put bytestring into a byte string register
    #[cfg_attr(feature = "std", display("put\t\ts16[{0}],{1}"))]
    Put(/** `s` register index */ u8, Blob),

    /// Move bytestring value between registers
    #[cfg_attr(feature = "std", display("mov\t\ts16[{0}],s16[{1}]"))]
    Mov(/** `s` register index */ u8, /** `s` register index */ u8),

    /// Swap bytestring value between registers
    #[cfg_attr(feature = "std", display("swp\t\ts16[{0}],s16[{1}]"))]
    Swp(/** `s` register index */ u8, /** `s` register index */ u8),

    /// Fill segment of bytestring with specific byte value
    #[cfg_attr(feature = "std", display("fill\ts16[{0}],{1}..{2},{3}"))]
    Fill(
        /** `s` register index */ u8,
        /** from */ u16,
        /** to */ u16,
        /** value */ u8,
    ),

    /// Put length of the string into `a16[0]` register
    #[cfg_attr(feature = "std", display("len\t\ts16[{0}],a16[0]"))]
    LenS(/** `s` register index */ u8),

    /// Count number of byte occurrences within the string and stores
    /// that value in `a16[0]`
    #[cfg_attr(feature = "std", display("count\ts16[{0}],{1},a16[0]"))]
    Count(/** `s` register index */ u8, /** byte to count */ u8),

    /// Compare two strings from two registers, putting result into `cm0`
    #[cfg_attr(feature = "std", display("cmp\t\ts16[{0}],s16[{0}]"))]
    Cmp(u8, u8),

    /// Compute length of the fragment shared between two strings
    #[cfg_attr(feature = "std", display("comm\ts16[{0}],s16[{1}]"))]
    Comm(u8, u8),

    /// Count number of occurrences of one string within another putting
    /// result to `a16[0]`
    #[cfg_attr(feature = "std", display("find\ts16[{0}],s16[{1}],a16[0]"))]
    Find(
        /** `s` register with string */ u8,
        /** `s` register with matching fragment */ u8,
    ),

    /// Extract byte string slice into an arithmetic register
    #[cfg_attr(feature = "std", display("extr\ts16[{0}],{1},a{2}{3}"))]
    ExtrA(/** `s` register index */ u8, /** offset */ u16, RegA, Reg32),

    /// Extract byte string slice into a non-arithmetic register
    #[cfg_attr(feature = "std", display("extr\ts16[{0}],{1},r{2}{3}"))]
    ExtrR(/** `s` register index */ u8, /** offset */ u16, RegR, Reg32),

    /// Join bytestrings from two registers
    #[cfg_attr(feature = "std", display("join\ts16[{0}],s16[{1}],s16[{2}]"))]
    Join(
        /** Source 1 */ u8,
        /** Source 2 */ u8,
        /** Destination */ u8,
    ),

    /// Split bytestring at a given index into two registers
    #[cfg_attr(
        feature = "std",
        display("split\ts16[{0}],{1},s16[{2}],s16[{3}]")
    )]
    Split(
        /** Source */ u8,
        /** Offset */ u16,
        /** Destination 1 */ u8,
        /** Destination 2 */ u8,
    ),

    /// Insert value from one of bytestring register at a given index of other
    /// bytestring register, shifting string bytes. If destination register
    /// does not fits the length of the new string, its final bytes are
    /// removed.
    #[cfg_attr(feature = "std", display("ins\t\ts16[{0}],s16[{1}],{2}"))]
    Ins(
        /** Insert from register */ u8,
        /** Insert to register */ u8,
        /** Offset for insert place */ u16,
    ),

    /// Delete bytes in a given range, shifting the remaining bytes
    #[cfg_attr(feature = "std", display("ins\t\ts16[{0}],{1}..{2}"))]
    Del(
        /** Register index */ u8,
        /** Delete from */ u16,
        /** Delete to */ u16,
    ),

    /// Extract fragment of bytestring into a register
    #[cfg_attr(feature = "std", display("transl\ts16[{0}],{1}..{2},s16[{3}]"))]
    Transl(
        /** Source */ u8,
        /** Start from */ u16,
        /** End at */ u16,
        /** Index to put translocated portion */ u8,
    ),
}

impl InstructionSet for BytesOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

/// Cryptographic hashing functions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[non_exhaustive]
pub enum DigestOp {
    /// Computes RIPEMD160 hash value
    #[cfg_attr(feature = "std", display("ripemd\ts16{0},r160{1},{2}"))]
    Ripemd(
        /** Index of string register */ Reg32,
        /** Which of `a16` registers contain start offset */ Reg32,
        /** Index of `r160` register to save result to */ Reg32,
        /** Clear string register after operation */ bool,
    ),

    /// Computes SHA256 hash value
    #[cfg_attr(feature = "std", display("sha2\ts16{0},r256{1},{2}"))]
    Sha2(
        /** Index of string register */ Reg32,
        /** Which of `a16` registers contain start offset */ Reg32,
        /** Index of `r256` register to save result to */ Reg32,
        /** Clear string register after operation */ bool,
    ),
}

impl InstructionSet for DigestOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

/// Operations on Secp256k1 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum SecpOp {
    /// Generates new elliptic curve point value saved into destination
    /// register in `r512` set using scalar value from the source `r256`
    /// register
    #[cfg_attr(feature = "std", display("secpgen\tr256{0},r512{1}"))]
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),

    /// Multiplies elliptic curve point on a scalar
    #[cfg_attr(feature = "std", display("secpmul\t{0}256{1},r512{2},r512{3}"))]
    Mul(
        /** Use `a` or `r` register as scalar source */ RegBlock,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),

    /// Adds two elliptic curve points
    #[cfg_attr(
        feature = "std",
        display("secpadd\tr512{0},r512{1},r512{2},{3}")
    )]
    Add(
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
        /** Allow overflows */ bool,
    ),

    /// Negates elliptic curve point
    #[cfg_attr(feature = "std", display("secpneg\tr512{0},r512{1}"))]
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}

impl InstructionSet for SecpOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

/// Operations on Curve25519 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum Curve25519Op {
    /// Generates new elliptic curve point value saved into destination
    /// register in `r512` set using scalar value from the source `r256`
    /// register
    #[cfg_attr(feature = "std", display("edgen\tr256{0},r512{1}"))]
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),

    /// Multiplies elliptic curve point on a scalar
    #[cfg_attr(feature = "std", display("edmul\t{0}256{1},r512{2},r512{3}"))]
    Mul(
        /** Use `a` or `r` register as scalar source */ RegBlock,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),

    /// Adds two elliptic curve points
    #[cfg_attr(feature = "std", display("edadd\tr512{0},r512{1},r512{2},{3}"))]
    Add(
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
        /** Allow overflows */ bool,
    ),

    /// Negates elliptic curve point
    #[cfg_attr(feature = "std", display("edneg\tr512{0},r512{1}"))]
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}

impl InstructionSet for Curve25519Op {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}
