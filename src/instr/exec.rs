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

use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, BitXor, Neg, Rem, Shl, Shr};

use bitcoin_hashes::{ripemd160, sha256, sha512, Hash};

use super::{
    ArithmeticOp, BitwiseOp, Bytecode, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp,
    Instr, MoveOp, NOp, PutOp, Secp256k1Op,
};
use crate::instr::{FloatEqFlag, IntFlags, MergeFlag, SignFlag};
use crate::reg::{MaybeNumber, Number, RegisterSet, Registers};
use crate::{LibSite, RegR};

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
            PutOp::ClrA(reg, index) => regs.set(reg, index, MaybeNumber::none()),
            PutOp::ClrF(reg, index) => regs.set(reg, index, MaybeNumber::none()),
            PutOp::ClrR(reg, index) => regs.set(reg, index, MaybeNumber::none()),
            PutOp::PutA(reg, index, number) => regs.set(reg, index, number),
            PutOp::PutF(reg, index, number) => regs.set(reg, index, number),
            PutOp::PutR(reg, index, number) => regs.set(reg, index, number),
            PutOp::PutIfA(reg, index, number) => regs.set_if(reg, index, number),
            PutOp::PutIfR(reg, index, number) => regs.set_if(reg, index, number),
        };
        ExecStep::Next
    }
}

impl InstructionSet for MoveOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            MoveOp::MovA(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
                regs.set(reg, idx1, MaybeNumber::none());
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
                regs.set(reg, idx1, MaybeNumber::none());
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
                regs.set(reg, idx1, MaybeNumber::none());
            }
            MoveOp::DupR(reg, idx1, idx2) => {
                regs.set(reg, idx2, regs.get(reg, idx1));
            }

            MoveOp::CpyA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::CnvA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout().into_signed());
                regs.set(dreg, didx, val);
            }
            MoveOp::CnvF(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::CpyR(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                let mut val1 = regs.get(sreg, sidx);
                let mut val2 = regs.get(dreg, didx);
                regs.st0 = val1.reshape(dreg.layout()) && val2.reshape(sreg.layout());
                regs.set(dreg, didx, val1);
                regs.set(sreg, sidx, val2);
            }
            MoveOp::CnvAF(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set(dreg, didx, val);
            }
            MoveOp::CnvFA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get(sreg, sidx);
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
            CmpOp::GtA(sign_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    val1.applying_sign(sign_flag).cmp(&val2.applying_sign(sign_flag))
                }) == Some(Ordering::Greater);
            }
            CmpOp::GtF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    if eq_flag == FloatEqFlag::Rounding {
                        val1.rounding_cmp(&val2)
                    } else {
                        val1.cmp(&val2)
                    }
                }) == Some(Ordering::Greater);
            }
            CmpOp::GtR(reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| val1.cmp(&val2))
                    == Some(Ordering::Greater);
            }
            CmpOp::LtA(sign_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    val1.applying_sign(sign_flag).cmp(&val2.applying_sign(sign_flag))
                }) == Some(Ordering::Less);
            }
            CmpOp::LtF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    if eq_flag == FloatEqFlag::Rounding {
                        val1.rounding_cmp(&val2)
                    } else {
                        val1.cmp(&val2)
                    }
                }) == Some(Ordering::Less);
            }
            CmpOp::LtR(reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| val1.cmp(&val2))
                    == Some(Ordering::Less);
            }
            CmpOp::EqA(st, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_both(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| val1 == val2)
                    .unwrap_or(st);
            }
            CmpOp::EqF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_both(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| {
                        if eq_flag == FloatEqFlag::Rounding {
                            val1.rounding_eq(&val2)
                        } else {
                            val1 == val2
                        }
                    })
                    .unwrap_or(false);
            }
            CmpOp::EqR(st, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_both(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| val1 == val2)
                    .unwrap_or(st);
            }
            CmpOp::IfZA(reg, idx) => {
                regs.st0 = regs.get(reg, idx).map(Number::is_zero).unwrap_or(false)
            }
            CmpOp::IfZR(reg, idx) => {
                regs.st0 = regs.get(reg, idx).map(Number::is_zero).unwrap_or(false)
            }
            CmpOp::IfNA(reg, idx) => regs.st0 = regs.get(reg, idx).is_none(),
            CmpOp::IfNR(reg, idx) => regs.st0 = regs.get(reg, idx).is_none(),
            CmpOp::St(merge_flag, reg, idx) => {
                let st = Number::from(regs.st0 as u8);
                let res = match (*regs.get(reg, idx), merge_flag) {
                    (None, _) | (_, MergeFlag::Set) => st,
                    (Some(val), MergeFlag::Add) => {
                        val.int_add(st, IntFlags { signed: false, wrap: false }).unwrap_or(val)
                    }
                    (Some(val), MergeFlag::And) => val & st,
                    (Some(val), MergeFlag::Or) => val | st,
                };
                regs.set(reg, idx, Some(res));
            }
            CmpOp::StInv => {
                regs.st0 = !regs.st0;
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for ArithmeticOp {
    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        let is_some = match self {
            ArithmeticOp::Abs(reg, idx) => regs.set(reg, idx, regs.get(reg, idx).map(Number::abs)),
            ArithmeticOp::AddA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_add(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::AddF(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_add(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::SubA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_sub(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::SubF(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_sub(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::MulA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_mul(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::MulF(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_mul(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::DivA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_div(val2, flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::DivF(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_div(val2, flags));
                regs.set(reg, srcdst, res) && !res.map(Number::is_nan).unwrap_or(false)
            }
            ArithmeticOp::Rem(reg1, idx1, reg2, idx2, regd, idxd) => {
                let res =
                    regs.get_both(reg1, idx1, reg2, idx2).and_then(|(val1, val2)| val1.rem(val2));
                regs.set(regd, idxd, res)
            }
            ArithmeticOp::Stp(reg, idx, step) => regs.set(
                reg,
                idx,
                regs.get(reg, idx).and_then(|val| {
                    val.int_add(Number::from(step), IntFlags { signed: false, wrap: false })
                }),
            ),
            ArithmeticOp::Neg(reg, idx) => regs.set(reg, idx, regs.get(reg, idx).map(Number::neg)),
        };
        regs.st0 = is_some;
        ExecStep::Next
    }
}

impl InstructionSet for BitwiseOp {
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep {
        match self {
            BitwiseOp::And(reg, src1, src2, dst) => {
                regs.op(reg, src1, reg, src2, reg, dst, BitAnd::bitand)
            }
            BitwiseOp::Or(reg, src1, src2, dst) => {
                regs.op(reg, src1, reg, src2, reg, dst, BitOr::bitor)
            }
            BitwiseOp::Xor(reg, src1, src2, dst) => {
                regs.op(reg, src1, reg, src2, reg, dst, BitXor::bitxor)
            }
            BitwiseOp::Not(reg, idx) => {
                regs.set(reg, idx, !regs.get(reg, idx));
            }
            BitwiseOp::Shl(reg1, shift, reg2, srcdst) => {
                regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Shl::shl)
            }
            BitwiseOp::ShrA(flag, reg1, shift, reg2, srcdst) => {
                let res = regs.get_both(reg1, shift, reg2, srcdst).map(|(shift, val)| {
                    if flag == SignFlag::Unsigned {
                        val.shr(shift)
                    } else {
                        val.shr_signed(shift)
                    }
                });
                regs.set(reg2, srcdst, res);
            }
            BitwiseOp::ShrR(reg1, shift, reg2, srcdst) => {
                regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Shr::shr)
            }
            BitwiseOp::Scl(reg1, shift, reg2, srcdst) => {
                regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Number::scl)
            }
            BitwiseOp::Scr(reg1, shift, reg2, srcdst) => {
                regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Number::scr)
            }
            BitwiseOp::RevA(reg, idx) => {
                regs.set(reg, idx, regs.get(reg, idx).map(Number::reverse_bits));
            }
            BitwiseOp::RevR(reg, idx) => {
                regs.set(reg, idx, regs.get(reg, idx).map(Number::reverse_bits));
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for BytesOp {
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep { todo!() }
}

impl InstructionSet for DigestOp {
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep {
        match self {
            DigestOp::Ripemd(src, dst) => {
                regs.set(
                    RegR::R160,
                    dst,
                    regs.get_s(src).map(|s| ripemd160::Hash::hash(&s).into_inner()),
                );
            }
            DigestOp::Sha256(src, dst) => {
                regs.set(
                    RegR::R256,
                    dst,
                    regs.get_s(src).map(|s| sha256::Hash::hash(&s).into_inner()),
                );
            }
            DigestOp::Sha512(src, dst) => {
                regs.set(
                    RegR::R512,
                    dst,
                    regs.get_s(src).map(|s| sha512::Hash::hash(&s).into_inner()),
                );
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for Secp256k1Op {
    #[cfg(not(feature = "secp256k1"))]
    fn exec(self, _: &mut Registers, _: LibSite) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Secp256k1 instructions")
    }

    #[cfg(feature = "secp256k1")]
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep {
        use secp256k1::{PublicKey, SecretKey};

        use crate::RegA;
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
    fn exec(self, _: &mut Registers, _: LibSite) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Curve25519 instructions")
    }

    #[cfg(feature = "curve25519")]
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep { todo!() }
}

impl InstructionSet for NOp {
    fn exec(self, _: &mut Registers, _: LibSite) -> ExecStep { ExecStep::Next }
}
