// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::u5;

use crate::registers::{Reg, Reg32, Reg8, RegA, RegR, Registers};
use crate::{Blob, LibSite};

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

/// Trait for instructions
pub trait Instruction {
    /// Executes given instruction taking all registers as input and output.
    /// The method is provided with the current code position which may be
    /// used by the instruction for constructing call stack.
    ///
    /// Returns whether further execution should be stopped.
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep;

    /// Returns length of the instruction block in bytes
    fn len(self) -> u16;
}

/// Full set of instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// #[cfg_attr(feature = "std", derive(Display), display(inner))]
#[non_exhaustive]
pub enum Instr<Extension>
where
    Extension: Instruction,
{
    /// Control-flow instructions
    // #[value = 0b00_000_000]
    ControlFlow(ControlFlowOp),

    /// Instructions setting register values
    // #[value = 0b00_001_000]
    Put(PutOp),

    /// Instructions moving and swapping register values
    // #[value = 0b00_010_000]
    Move(MoveOp),

    /// Instructions comparing register values
    // #[value = 0b00_011_000]
    Cmp(CmpOp),

    /// Arithmetic instructions
    // #[value = 0b00_100_000]
    Arithmetic(ArithmeticOp),

    /// Bit operations & boolean algebra instructions
    // #[value = 0b00_101_000]
    Bitwise(BitwiseOp),

    /// Operations on byte strings
    // #[value = 0b00_110_000]
    Bytes(BytesOp),

    /// Cryptographic hashing functions
    // #[value = 0b01_000_000]
    Digest(DigestOp),

    /// Operations on Secp256k1 elliptic curve
    // #[value = 0b01_001_000]
    Secp256k1(SecpOp),

    /// Operations on Curve25519 elliptic curve
    // #[value = 0b01_001_100]
    Curve25519(Curve25519Op),

    /// Reserved operations which can be provided by a host environment
    // #[value = 0b10_000_000]
    ExtensionCodes(Extension),

    /// No-operation instruction
    // #[value = 0b11_111_111]
    Nop,
}

impl<Extension> Instruction for Instr<Extension>
where
    Extension: Instruction,
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

    fn len(self) -> u16 {
        match self {
            Instr::ControlFlow(instr) => instr.len(),
            Instr::Put(instr) => instr.len(),
            Instr::Move(instr) => instr.len(),
            Instr::Cmp(instr) => instr.len(),
            Instr::Arithmetic(instr) => instr.len(),
            Instr::Bitwise(instr) => instr.len(),
            Instr::Bytes(instr) => instr.len(),
            Instr::Digest(instr) => instr.len(),
            Instr::Secp256k1(instr) => instr.len(),
            Instr::Curve25519(instr) => instr.len(),
            Instr::ExtensionCodes(instr) => instr.len(),
            Instr::Nop => 1,
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
    // #[value = 0b000]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program
    /// success)
    #[cfg_attr(feature = "std", display("succ"))]
    // #[value = 0b001]
    Succ,

    /// Unconditionally jumps to an offset. Increments `cy0`.
    #[cfg_attr(feature = "std", display("jmp\t{0:#06X}"))]
    // #[value = 0b010]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments
    /// `cy0`.
    #[cfg_attr(feature = "std", display("jif\t{0:#06X}"))]
    // #[value = 0b011]
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

impl Instruction for ControlFlowOp {
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

    fn len(self) -> u16 {
        match self {
            ControlFlowOp::Fail => 1,
            ControlFlowOp::Succ => 1,
            ControlFlowOp::Jmp(_) => 3,
            ControlFlowOp::Jif(_) => 3,
            ControlFlowOp::Routine(_) => 3,
            ControlFlowOp::Call(_) => 3 + 32,
            ControlFlowOp::Exec(_) => 3 + 32,
            ControlFlowOp::Ret => 1,
        }
    }
}

/// Instructions setting register values
#[cfg_attr(feature = "std", derive(Display))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PutOp {
    /// Sets `a` register value to zero
    #[display("zero\t{0}{1}")]
    ZeroA(RegA, Reg32),

    /// Sets `r` register value to zero
    #[display("zero\t{0}{1}")]
    ZeroR(RegR, Reg32),

    /// Cleans a value of `a` register (sets it to undefined state)
    #[display("cl\t{0}{1}")]
    ClA(RegA, Reg32),

    /// Cleans a value of `r` register (sets it to undefined state)
    #[display("cl\t{0}{1}")]
    ClR(RegR, Reg32),

    /// Unconditionally assigns a value to `a` register
    #[display("put\t{0}{1}, {2}")]
    PutA(RegA, Reg32, Blob),

    /// Unconditionally assigns a value to `r` register
    #[display("put\t{0}{1}, {2}")]
    PutR(RegR, Reg32, Blob),

    /// Conditionally assigns a value to `a` register if the register is in
    /// uninitialized state
    #[display("putif\t{0}{1}, {2}")]
    PutAIf(RegA, Reg32, Blob),

    /// Conditionally assigns a value to `r` register if the register is in
    /// uninitialized state
    #[display("putif\t{0}{1}, {2}")]
    PutRIf(RegR, Reg32, Blob),
}

impl Instruction for PutOp {
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
            PutOp::PutAIf(reg, index, blob) => {
                regs.get(Reg::A(reg), index).or_else(|| {
                    regs.set(Reg::A(reg), index, Some(blob));
                    Some(blob)
                });
            }
            PutOp::PutRIf(reg, index, blob) => {
                regs.get(Reg::R(reg), index).or_else(|| {
                    regs.set(Reg::R(reg), index, Some(blob));
                    Some(blob)
                });
            }
        }
        ExecStep::Next
    }

    fn len(self) -> u16 {
        match self {
            PutOp::ZeroA(_, _)
            | PutOp::ZeroR(_, _)
            | PutOp::ClA(_, _)
            | PutOp::ClR(_, _) => 2,
            PutOp::PutA(_, _, Blob { len, .. })
            | PutOp::PutR(_, _, Blob { len, .. })
            | PutOp::PutAIf(_, _, Blob { len, .. })
            | PutOp::PutRIf(_, _, Blob { len, .. }) => 4u16.saturating_add(len),
        }
    }
}

/// Integer arithmetic types
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum NumType {
    /// Unsigned integer
    #[display("u")]
    Unsigned,

    /// Signed integer
    #[display("s")]
    Signed,

    /// Float number with 23-bit mantissa
    #[display("f")]
    Float23,

    /// Float number with 52 bit mantissa
    #[display("d")]
    Float52,
}

/// Instructions moving and swapping register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// #[cfg_attr(feature = "std", derive(Display))]
pub enum MoveOp {
    /// Swap operation. If the value does not fit destination bit dimensions
    /// truncates the most significant bits until they fit.
    SwpA(RegA, Reg32, RegA, Reg32),
    SwpR(RegR, Reg32, RegR, Reg32),
    Swp(RegA, Reg32, RegR, Reg32),

    /// Duplicates values of all register set into another set
    Mov(RegA, RegA, NumType),

    /// Duplicates value of one of the registers into another register
    MovA(RegA, Reg32, RegA, Reg32),
    MovR(RegA, Reg32, RegA, Reg32),
    MovAR(RegA, Reg32, RegR, Reg32),
    MovRA(RegR, Reg32, RegA, Reg32),
}

impl Instruction for MoveOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        match self {
            MoveOp::SwpA(_, _, _, _) => {}
            MoveOp::SwpR(_, _, _, _) => {}
            MoveOp::Swp(_, _, _, _) => {}
            MoveOp::Mov(_, _, _) => {}
            MoveOp::MovA(_, _, _, _) => {}
            MoveOp::MovR(_, _, _, _) => {}
            MoveOp::MovAR(_, _, _, _) => {}
            MoveOp::MovRA(_, _, _, _) => {}
        }
        ExecStep::Next
    }

    fn len(self) -> u16 {
        match self {
            MoveOp::SwpA(_, _, _, _)
            | MoveOp::SwpR(_, _, _, _)
            | MoveOp::Swp(_, _, _, _) => 3,
            MoveOp::Mov(_, _, _) => 2,
            MoveOp::MovA(_, _, _, _)
            | MoveOp::MovR(_, _, _, _)
            | MoveOp::MovAR(_, _, _, _)
            | MoveOp::MovRA(_, _, _, _) => 3,
        }
    }
}

/// Instructions comparing register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CmpOp {
    /// Compares value of two arithmetic (`A`) registers setting `st0` to
    /// `true` if the first parameter is greater (and not equal) than the
    /// second one
    // #[value = 0b110] // 3 + 5 + 3 + 5 => 16 bits
    Gt(RegA, Reg32, RegA, Reg32),

    /// Compares value of two non-arithmetic (`R`) registers setting `st0` to
    /// `true` if the first parameter is less (and not equal) than the second
    /// one
    // #[value = 0b111]
    Lt(RegR, Reg32, RegR, Reg32),

    /// Checks equality of value in two arithmetic (`A`) registers putting
    /// result into `st0`
    // #[value = 0b100]
    Eqa(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting
    /// result into `st0`
    // #[value = 0b101]
    Eqr(RegR, Reg32, RegR, Reg32),

    /// Measures bit length of a value in one fo the registers putting result
    /// to `a16[0]`
    Len(RegA, Reg32),

    /// Counts number of `1` bits in register putting result to `a16[0]`
    /// register.
    Cnt(RegA, Reg32),

    /// Assigns value of `a8[0]` register to `st0`
    St2A,

    /// `st0` value of `st0` register to the result of `a8[0] == 1`
    A2St,
}

impl Instruction for CmpOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}

/// Variants of arithmetic operations
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Arithmetics {
    IntChecked(bool),
    IntUnchecked(bool),
    IntArbitraryPrecision(bool),
    Float,
    FloatArbitraryPrecision,
}

/// Arithmetic instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ArithmeticOp {
    Neg(RegA, Reg32),                     // 3 + 5 = 8 bits
    Inc(Arithmetics, RegA, Reg32, u5),    // Increases value on a given step
    Add(Arithmetics, RegA, Reg32, Reg32), // 3 + 3 + 5 + 5  => 16 bits
    Sub(Arithmetics, RegA, Reg32, Reg32),
    Mul(Arithmetics, RegA, Reg32, Reg32),
    Div(Arithmetics, RegA, Reg32, Reg32),
    Mod(RegA, Reg32),              // 3 + 5 = 8 bits
    Abs(RegA, Reg32, RegA, Reg32), // 3 + 5 + 3 + 5 => 16 bits
}

impl Instruction for ArithmeticOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}

/// Bit operations & boolean algebra instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum BitwiseOp {
    And(
        RegA,
        Reg32,
        Reg32,
        /// Operation destination, only first 8 registers
        Reg8,
    ),
    Or(RegA, Reg32, Reg32, Reg8),
    Xor(RegA, Reg32, Reg32, Reg8),

    Not(RegA, Reg32),

    Shl(RegA, Reg32, Reg32 /* Always `a8` */, Reg8),
    Shr(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle left
    Scl(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle right
    Scr(RegA, Reg32, Reg32, Reg8),
}

impl Instruction for BitwiseOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}

/// Operations on byte strings
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum BytesOp {
    Puts(/** `s` register index */ u8, u16, [u8; u16::MAX as usize]),

    Movs(/** `s` register index */ u8, /** `s` register index */ u8),

    Swps(/** `s` register index */ u8, /** `s` register index */ u8),

    Fill(
        /** `s` register index */ u8,
        /** from */ u16,
        /** to */ u16,
        /** value */ u8,
    ),

    /// Returns length of the string
    Lens(/** `s` register index */ u8),

    /// Counts number of byte occurrences within the string
    Counts(/** `s` register index */ u8, /** byte to count */ u8),

    /// Compares two strings from two registers, putting result into `cm0`
    Cmps(u8, u8),

    /// Computes length of the fragment shared between two strings
    Common(u8, u8),

    /// Counts number of occurrences of one string within another putting
    /// result to `a16[0]`
    Find(
        /** `s` register with string */ u8,
        /** `s` register with matching fragment */ u8,
    ),

    /// Extracts value into a register
    Exta(RegA, Reg32, /** `s` register index */ u8, /** offset */ u16),
    Extr(RegR, Reg32, /** `s` register index */ u8, /** offset */ u16),

    Join(
        /** Source 1 */ u8,
        /** Source 2 */ u8,
        /** Destination */ u8,
    ),
    Split(
        /** Source */ u8,
        /** Offset */ u16,
        /** Destination 1 */ u8,
        /** Destination 2 */ u8,
    ),
    Ins(
        /** Insert from register */ u8,
        /** Insert to register */ u8,
        /** Offset for insert place */ u16,
    ),
    Del(
        /** Register index */ u8,
        /** Delete from */ u16,
        /** Delete to */ u16,
    ),
    /// Translocates fragment of bytestring into a register
    Transl(
        /** Source */ u8,
        /** Start from */ u16,
        /** End at */ u16,
        /** Index to put translocated portion */ u8,
    ),
}

impl Instruction for BytesOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}

/// Cryptographic hashing functions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum DigestOp {
    Ripemd(
        /** Which of `a16` registers contain start offset */ Reg32,
        /** Index of string register */ Reg32,
        /** Index of `r160` register to save result to */ Reg32,
        /** Clear string register after operation */ bool,
    ),
    Sha2(
        /** Which of `a16` registers contain start offset */ Reg32,
        /** Index of string register */ Reg32,
        /** Index of `r160` register to save result to */ Reg32,
        /** Clear string register after operation */ bool,
    ),
}

impl Instruction for DigestOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}

/// Operations on Secp256k1 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SecpOp {
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),
    Mul(
        /** Use `a` or `r` register as scalar source */ bool,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),
    Add(
        /** Allow overflows */ bool,
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
    ),
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}

impl Instruction for SecpOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}

/// Operations on Curve25519 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Curve25519Op {
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),
    Mul(
        /** Use `a` or `r` register as scalar source */ bool,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),
    Add(
        /** Allow overflows */ bool,
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
    ),
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}

impl Instruction for Curve25519Op {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }

    fn len(self) -> u16 {
        todo!()
    }
}
