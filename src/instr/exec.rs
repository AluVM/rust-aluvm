// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

use super::{
    ArithmeticOp, BitwiseOp, Bytecode, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp,
    Instr, MoveOp, NOp, NumType, PutOp, SecpOp,
};
use crate::reg::{Reg32, RegVal, Registers};
use crate::LibSite;

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
pub trait InstructionSet: Bytecode + std::fmt::Display {
    /// Executes given instruction taking all registers as input and output.
    /// The method is provided with the current code position which may be
    /// used by the instruction for constructing call stack.
    ///
    /// Returns whether further execution should be stopped.
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep;
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
            ControlFlowOp::Ret => regs.ret().map(ExecStep::Call).unwrap_or(ExecStep::Stop),
        }
    }
}

impl InstructionSet for PutOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            PutOp::ZeroA(reg, index) => regs.set(reg, index, Some(0.into())),
            PutOp::ZeroR(reg, index) => regs.set(reg, index, Some(0.into())),
            PutOp::ClA(reg, index) => regs.set(reg, index, None),
            PutOp::ClR(reg, index) => regs.set(reg, index, None),
            PutOp::PutA(reg, index, blob) => regs.set(reg, index, Some(blob)),
            PutOp::PutR(reg, index, blob) => regs.set(reg, index, Some(blob)),
            PutOp::PutIfA(reg, index, blob) => regs.set_if(reg, index, blob),
            PutOp::PutIfR(reg, index, blob) => regs.set_if(reg, index, blob),
        }
        ExecStep::Next
    }
}

impl InstructionSet for MoveOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            MoveOp::SwpA(reg1, index1, reg2, index2) => {
                regs.set(reg1, index1, regs.get(reg2, index2));
                regs.set(reg2, index2, regs.get(reg1, index1));
            }
            MoveOp::SwpR(reg1, index1, reg2, index2) => {
                regs.set(reg1, index1, regs.get(reg2, index2));
                regs.set(reg2, index2, regs.get(reg1, index1));
            }
            MoveOp::SwpAR(reg1, index1, reg2, index2) => {
                regs.set(reg1, index1, regs.get(reg2, index2));
                regs.set(reg2, index2, regs.get(reg1, index1));
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
                regs.set(dreg, didx, regs.get(sreg, sidx));
            }
            MoveOp::MovR(sreg, sidx, dreg, didx) => {
                regs.set(dreg, didx, regs.get(sreg, sidx));
            }
            MoveOp::MovAR(sreg, sidx, dreg, didx) => {
                regs.set(dreg, didx, regs.get(sreg, sidx));
            }
            MoveOp::MovRA(sreg, sidx, dreg, didx) => {
                regs.set(dreg, didx, regs.get(sreg, sidx));
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for CmpOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        // TODO: Implement comparison operations
        ExecStep::Next
    }
}

impl InstructionSet for ArithmeticOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            ArithmeticOp::Neg(reg, index) => {
                regs.get(reg, index).map(|mut blob| {
                    blob.bytes[reg as usize] = 0xFF ^ blob.bytes[reg as usize];
                    regs.set(reg, index, Some(blob));
                });
            }
            ArithmeticOp::Stp(dir, arithm, reg, index, step) => {
                regs.op1(
                    reg,
                    index,
                    arithm.is_ap(),
                    Reg32::Reg1,
                    RegVal::step_op(arithm, *step as i8 * dir.multiplier()),
                );
            }
            ArithmeticOp::Add(arithm, reg, src1, src2) => {
                regs.op2(
                    reg,
                    src1,
                    src2,
                    arithm.is_ap(),
                    Reg32::Reg1,
                    RegVal::add_op(arithm),
                );
            }
            ArithmeticOp::Sub(arithm, reg, src1, src2) => {
                regs.op2(
                    reg,
                    src1,
                    src2,
                    arithm.is_ap(),
                    Reg32::Reg1,
                    RegVal::sub_op(arithm),
                );
            }
            ArithmeticOp::Mul(arithm, reg, src1, src2) => {
                regs.op2(
                    reg,
                    src1,
                    src2,
                    arithm.is_ap(),
                    Reg32::Reg1,
                    RegVal::mul_op(arithm),
                );
            }
            ArithmeticOp::Div(arithm, reg, src1, src2) => {
                regs.op2(
                    reg,
                    src1,
                    src2,
                    arithm.is_ap(),
                    Reg32::Reg1,
                    RegVal::div_op(arithm),
                );
            }
            ArithmeticOp::Mod(reg1, index1, reg2, index2, reg3, index3) => {}
            ArithmeticOp::Abs(reg, index) => {}
        }
        ExecStep::Next
    }
}

impl InstructionSet for BitwiseOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        match self {
            BitwiseOp::And(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, BitAnd::bitand),
            BitwiseOp::Or(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, BitOr::bitor),
            BitwiseOp::Xor(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, BitXor::bitxor),
            BitwiseOp::Not(reg, idx) => regs.set(reg, idx, !regs.get(reg, idx)),
            BitwiseOp::Shl(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, Shl::shl),
            BitwiseOp::Shr(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, Shr::shr),
            BitwiseOp::Scl(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, RegVal::scl),
            BitwiseOp::Scr(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, RegVal::scr),
        }
        ExecStep::Next
    }
}

impl InstructionSet for BytesOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

impl InstructionSet for DigestOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

impl InstructionSet for SecpOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

impl InstructionSet for Curve25519Op {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
    }
}

impl InstructionSet for NOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        ExecStep::Next
    }
}
