// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

use super::{
    ArithmeticOp, BitwiseOp, Bytecode, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp,
    Instr, MoveOp, NOp, PutOp, Secp256k1Op,
};
use crate::reg::{Number, Reg32, RegVal, RegisterSet, Registers};
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

/// Trait for instructions
pub trait InstructionSet: Bytecode + core::fmt::Display + core::fmt::Debug {
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
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.exec(regs, site),
            #[cfg(feature = "curve25519")]
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
            ControlFlowOp::Jmp(offset) => {
                regs.jmp().map(|_| ExecStep::Jump(offset)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Jif(offset) => {
                if regs.st0 {
                    regs.jmp().map(|_| ExecStep::Jump(offset)).unwrap_or(ExecStep::Stop)
                } else {
                    ExecStep::Next
                }
            }
            ControlFlowOp::Routine(offset) => {
                regs.call(site).map(|_| ExecStep::Jump(offset)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Call(site) => {
                regs.call(site).map(|_| ExecStep::Call(site)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Exec(site) => {
                regs.jmp().map(|_| ExecStep::Call(site)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Ret => regs.ret().map(ExecStep::Call).unwrap_or(ExecStep::Stop),
        }
    }
}

impl InstructionSet for PutOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            PutOp::ClrA(reg, index) => regs.set(reg, index, RegVal::none()),
            PutOp::ClrF(reg, index) => regs.set(reg, index, RegVal::none()),
            PutOp::ClrR(reg, index) => regs.set(reg, index, RegVal::none()),
            PutOp::PutA(reg, index, number) => regs.set(reg, index, number),
            PutOp::PutF(reg, index, number) => regs.set(reg, index, number),
            PutOp::PutR(reg, index, number) => regs.set(reg, index, number),
            PutOp::PutIfA(reg, index, number) => regs.set_if(reg, index, number),
            PutOp::PutIfR(reg, index, number) => regs.set_if(reg, index, number),
        }
        ExecStep::Next
    }
}

impl InstructionSet for MoveOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            MoveOp::MovA(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
                regs.set(reg, idx1, None);
            }
            MoveOp::DupA(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
            }
            MoveOp::SwpA(reg, idx1, idx2) => {
                let val = regs.get(reg, idx2);
                regs.set(reg, idx2, regs.get(reg, idx1));
                regs.set(reg, idx1, val);
            }
            MoveOp::MovF(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
                regs.set(reg, idx1, None);
            }
            MoveOp::DupF(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
            }
            MoveOp::SwpF(reg, idx1, idx2) => {
                let val = regs.get(reg, idx2);
                regs.set(reg, idx2, regs.get(reg, idx1));
                regs.set(reg, idx1, val);
            }
            MoveOp::MovR(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
                regs.set(reg, idx1, None);
            }
            MoveOp::DupR(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
            }

            MoveOp::CpyA(sreg, sidx, dreg, didx) => {
                let val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::CnvA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout().into_signed());
                regs.set(dreg, didx, val);
            }
            MoveOp::CnvF(sreg, sidx, dreg, didx) => {
                let val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::CpyR(sreg, sidx, dreg, didx) => {
                let val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                let val1 = regs.get(sreg, sidx);
                let val2 = regs.get(dreg, didx);
                regs.st0 = val1.reshape(dreg.layout()) && val2.reshape(sreg.layout());
                regs.set(dreg, didx, val1);
                regs.set(sreg, sidx, val2);
            }
            MoveOp::CnvAF(sreg, sidx, dreg, didx) => {
                let val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::CnvFA(sreg, sidx, dreg, didx) => {
                let val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for CmpOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            CmpOp::GtA(num_type, reg, idx1, idx2) => {
                regs.st0 =
                    RegVal::partial_cmp_op(num_type)(regs.get(reg, idx1), regs.get(reg, idx2))
                        == Some(Ordering::Greater);
            }
            CmpOp::GtR(reg1, idx1, reg2, idx2) => {
                regs.st0 = regs.get(reg1, idx1).partial_cmp_uint(regs.get(reg2, idx2))
                    == Some(Ordering::Greater);
            }
            CmpOp::LtA(num_type, reg, idx1, idx2) => {
                regs.st0 =
                    RegVal::partial_cmp_op(num_type)(regs.get(reg, idx1), regs.get(reg, idx2))
                        == Some(Ordering::Less);
            }
            CmpOp::LtR(reg1, idx1, reg2, idx2) => {
                regs.st0 = regs.get(reg1, idx1).partial_cmp_uint(regs.get(reg2, idx2))
                    == Some(Ordering::Less);
            }
            CmpOp::EqA(reg1, idx1, reg2, idx2) => {
                regs.st0 = regs.get(reg1, idx1) == regs.get(reg2, idx2);
            }
            CmpOp::EqR(reg1, idx1, reg2, idx2) => {
                regs.st0 = regs.get(reg1, idx1) == regs.get(reg2, idx2);
            }
            CmpOp::Len(reg, idx) => {
                regs.a16[0] = regs.get(reg, idx).map(|v| v.len);
            }
            CmpOp::Cnt(reg, idx) => {
                regs.a16[0] = regs.get(reg, idx).map(|v| v.count_ones());
            }
            CmpOp::St(_, _) => {
                regs.a8[0] = if regs.st0 { Some(1) } else { Some(0) };
            }
            CmpOp::A2St => {
                regs.st0 = regs.a8[1].map(|val| val != 0).unwrap_or(false);
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for ArithmeticOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            ArithmeticOp::Neg(reg, index) => {
                regs.get(reg, index).map(|mut blob| {
                    blob.bytes[reg as usize] ^= 0xFF;
                    regs.set(reg, index, Some(blob));
                });
            }
            ArithmeticOp::Stp(dir, arithm, reg, index, step) => {
                regs.op_ap1(
                    reg,
                    index,
                    arithm.is_ap(),
                    Reg32::Reg1,
                    Number::step_op(arithm, step.as_u8() as i8 * dir.multiplier()),
                );
            }
            ArithmeticOp::Add(arithm, reg, src1, src2) => {
                regs.op_ap2(reg, src1, src2, arithm.is_ap(), Reg32::Reg1, Number::add_op(arithm));
            }
            ArithmeticOp::Sub(arithm, reg, src1, src2) => {
                regs.op_ap2(reg, src1, src2, arithm.is_ap(), Reg32::Reg1, Number::sub_op(arithm));
            }
            ArithmeticOp::Mul(arithm, reg, src1, src2) => {
                regs.op_ap2(reg, src1, src2, arithm.is_ap(), Reg32::Reg1, Number::mul_op(arithm));
            }
            ArithmeticOp::Div(arithm, reg, src1, src2) => {
                regs.op_ap2(reg, src1, src2, arithm.is_ap(), Reg32::Reg1, Number::div_op(arithm));
            }
            ArithmeticOp::Rem(arithm, reg, src1, src2) => {
                regs.op_ap2(reg, src1, src2, arithm.is_ap(), Reg32::Reg1, Number::rem_op(arithm));
            }
            ArithmeticOp::Abs(reg, index) => {
                todo!()
            }
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
            BitwiseOp::Scl(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, Number::scl),
            BitwiseOp::Scr(reg, src1, src2, dst) => regs.op(reg, src1, src2, dst, Number::scr),
        }
        ExecStep::Next
    }
}

impl InstructionSet for BytesOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep { todo!() }
}

impl InstructionSet for DigestOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep { todo!() }
}

impl InstructionSet for Secp256k1Op {
    #[cfg(not(feature = "secp256k1"))]
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Secp256k1 instructions")
    }

    #[cfg(feature = "secp256k1")]
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        use secp256k1::{PublicKey, SecretKey};

        use crate::{RegA, RegR};
        match self {
            Secp256k1Op::Gen(src, dst) => {
                let res = regs
                    .get(RegA::A256, src)
                    .and_then(|src| SecretKey::from_slice(src.as_ref()).ok())
                    .map(|sk| PublicKey::from_secret_key(&regs.secp, &sk))
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set(RegR::R512, dst, res);
            }

            Secp256k1Op::Mul(block, scal, src, dst) => {
                let reg = block.into_reg(256).expect("register set does not match standard");
                let res = regs
                    .get(reg, scal)
                    .and_then(|scal| {
                        regs.get(RegR::R512, src)
                            .and_then(|val| PublicKey::from_slice(val.as_ref()).ok())
                            .map(|pk| (scal, pk))
                    })
                    .and_then(|(scal, mut pk)| {
                        pk.mul_assign(&regs.secp, scal.as_ref()).map(|_| pk).ok()
                    })
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set(RegR::R512, dst, res);
            }

            Secp256k1Op::Add(src, srcdst) => {
                let res = regs
                    .get(RegR::R512, src)
                    .and_then(|pk1| PublicKey::from_slice(pk1.as_ref()).ok())
                    .and_then(|pk1| regs.get(RegR::R512, srcdst).map(|pk2| (pk1, pk2)))
                    .and_then(|(mut pk1, pk2)| {
                        pk1.add_exp_assign(&regs.secp, pk2.as_ref()).map(|_| pk1).ok()
                    })
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set(RegR::R512, srcdst, res);
            }

            Secp256k1Op::Neg(src, dst) => {
                let res = regs
                    .get(RegR::R512, src)
                    .and_then(|pk| PublicKey::from_slice(pk.as_ref()).ok())
                    .map(|mut pk| {
                        pk.negate_assign(&regs.secp);
                        pk
                    })
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set(RegR::R512, dst, res);
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for Curve25519Op {
    #[cfg(not(feature = "curve25519"))]
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Curve25519 instructions")
    }

    #[cfg(feature = "curve25519")]
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep { todo!() }
}

impl InstructionSet for NOp {
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep { ExecStep::Next }
}
