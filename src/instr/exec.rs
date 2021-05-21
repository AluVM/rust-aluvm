// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::{u1024, u512};

use crate::instr::{
    ArithmeticOp, Arithmetics, BitwiseOp, Bytecode, BytesOp, CmpOp,
    ControlFlowOp, Curve25519Op, DigestOp, MoveOp, NOp, NumType, PutOp, SecpOp,
};
use crate::registers::{Reg, Reg32, RegA, Registers};
use crate::types::Value;
use crate::{Instr, LibSite};

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
            ControlFlowOp::Ret => {
                regs.ret().map(ExecStep::Call).unwrap_or(ExecStep::Stop)
            }
        }
    }
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

impl InstructionSet for BitwiseOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        todo!()
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
