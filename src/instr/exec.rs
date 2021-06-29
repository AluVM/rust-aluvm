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

use alloc::collections::BTreeSet;
use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, BitXor, Neg, Rem, Shl, Shr};

use bitcoin_hashes::{ripemd160, sha256, sha512, Hash};

use super::{
    ArithmeticOp, BitwiseOp, Bytecode, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp,
    Instr, MoveOp, NOp, PutOp, Secp256k1Op,
};
use crate::instr::{FloatEqFlag, IntFlags, MergeFlag, SignFlag};
use crate::reg::{MaybeNumber, Number, NumberLayout, RegisterSet, Registers};
use crate::{ByteStr, LibSite, Reg32, RegA, RegR};

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
    /// ISA IDs used by the provided instruction set.
    ///
    /// Each id must be up to 8 bytes and consist of upper case latin alphanumeric characters,
    /// starting with non-number.
    ///
    /// Concatenated length of the ISA IDs joined via '+' character must not exceed 64 bytes.
    fn ids() -> BTreeSet<&'static str>;

    /// Executes given instruction taking all registers as input and output.
    ///
    /// # Arguments
    ///
    /// The method is provided with the current code position which may be used by the instruction
    /// for constructing call stack.
    ///
    /// # Returns
    ///
    /// Returns whether further execution should be stopped.
    fn exec(self, regs: &mut Registers, site: LibSite) -> ExecStep;
}

impl<Extension> InstructionSet for Instr<Extension>
where
    Extension: InstructionSet,
{
    #[inline]
    fn ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert("ALU");
        set.extend(DigestOp::ids());
        set.extend(Secp256k1Op::ids());
        set.extend(Curve25519Op::ids());
        set
    }

    #[inline]
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
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

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
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn exec(self, regs: &mut Registers, _: LibSite) -> ExecStep {
        match self {
            PutOp::ClrA(reg, index) => regs.set(reg, index, MaybeNumber::none()),
            PutOp::ClrF(reg, index) => regs.set(reg, index, MaybeNumber::none()),
            PutOp::ClrR(reg, index) => regs.set(reg, index, MaybeNumber::none()),
            PutOp::PutA(reg, index, number) => regs.set(reg, index, *number),
            PutOp::PutF(reg, index, number) => regs.set(reg, index, *number),
            PutOp::PutR(reg, index, number) => regs.set(reg, index, *number),
            PutOp::PutIfA(reg, index, number) => regs.set_if(reg, index, *number),
            PutOp::PutIfR(reg, index, number) => regs.set_if(reg, index, *number),
        };
        ExecStep::Next
    }
}

impl InstructionSet for MoveOp {
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

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
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

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
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

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
            ArithmeticOp::Rem(reg1, idx1, reg2, idx2) => {
                let res =
                    regs.get_both(reg1, idx1, reg2, idx2).and_then(|(val1, val2)| val1.rem(val2));
                regs.set(reg2, idx2, res)
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
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

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
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[allow(warnings)]
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep {
        match self {
            BytesOp::Put(reg, bytes) => {
                regs.s16.insert(reg, *bytes);
            }
            BytesOp::Mov(reg1, reg2) => {
                regs.s16.remove(&reg1).and_then(|bs| regs.s16.insert(reg2, bs));
            }
            BytesOp::Swp(reg1, reg2) => {
                let bs1 = regs.s16.remove(&reg1);
                regs.s16.remove(&reg1).and_then(|bs2| regs.s16.insert(reg1, bs2));
                bs1.and_then(|bs1| regs.s16.insert(reg2, bs1));
            }
            BytesOp::Fill(reg, offset1, offset2, value, flag) => {
                let mut f = || -> Option<()> {
                    let o1 = regs.a16[offset1.to_usize()]?;
                    let o2 = regs.a16[offset2.to_usize()]?;
                    let range = o1..=o2;
                    let val = regs.a8[value.to_usize()]?;
                    let bs = regs.s16.entry(reg).or_insert(ByteStr::default());
                    if bs.len() <= *range.end() as usize && !flag {
                        return None;
                    }
                    bs.fill(o1..=o2, val);
                    Some(())
                };
                f().unwrap_or_else(|| regs.st0 = false);
            }
            BytesOp::Len(src, reg, dst) => {
                let mut f = || -> Option<()> {
                    let s = regs.get_s(src)?;
                    let len = s.len();
                    if !reg.int_layout().fits_usize(len) {
                        return None;
                    }
                    regs.set(reg, dst, len as u32);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(reg, dst, MaybeNumber::none());
                });
            }
            BytesOp::Cnt(src, byte, dst) => {
                let mut f = || -> Option<()> {
                    let val = regs.a8[byte as u8 as usize]?;
                    let bs = regs.s16.get(&src)?;
                    let count = bs.as_ref().into_iter().filter(|b| **b == val).count();
                    if !RegA::A16.int_layout().fits_usize(count) {
                        return None;
                    }
                    regs.set(RegA::A16, dst, count as u32);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(RegA::A16, dst, MaybeNumber::none());
                });
            }
            BytesOp::Eq(reg1, reg2) => {
                let s1 = regs.get_s(reg1);
                let s2 = regs.get_s(reg2);
                regs.st0 = match (s1, s2) {
                    (Some(s1), Some(s2)) => s1 == s2,
                    (None, None) => true,
                    _ => false,
                };
            }
            BytesOp::Find(reg1, reg2) => {
                let mut f = || -> Option<()> {
                    let (s1, s2) = regs.get_both_s(reg1, reg2)?;
                    let r1 = s1.bytes.as_ref();
                    let r2 = s2.bytes.as_ref();
                    let len = r2.len();
                    let mut count = 0usize;
                    for i in 0..r1.len() {
                        if r1[i..len] == r2[..len] {
                            count += 1;
                        }
                    }
                    if count > u16::MAX as usize {
                        regs.st0 = false;
                        count -= 1;
                    }
                    regs.set(RegA::A16, Reg32::Reg1, count as u16);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(RegA::A16, Reg32::Reg1, MaybeNumber::none());
                })
            }
            BytesOp::Rev(reg1, reg2) => {
                let mut f = || -> Option<()> {
                    let mut s = regs.get_s(reg1)?.clone();
                    let bs = s.as_mut();
                    bs.reverse();
                    regs.s16.insert(reg2, s);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(RegA::A16, Reg32::Reg1, MaybeNumber::none());
                })
            }
            BytesOp::Con(reg1, reg2, no, offset, len) => {
                todo!("(#6) complete bytestring opcode implementation")
            }
            BytesOp::Extr(src, dst, index, offset) => {
                let mut f = || -> Option<()> {
                    let s = regs.get_s(src)?.clone();
                    let offset = regs.a16[offset as u8 as usize]?;
                    let end = offset.saturating_add(dst.layout().bytes());
                    let num = Number::from_slice(&s.as_ref()[offset as usize..=end as usize]);
                    regs.set(dst, index, num);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(dst, index, MaybeNumber::none());
                })
            }
            BytesOp::Inj(src, dst, index, offset) => {
                let mut f = || -> Option<()> {
                    let mut s = regs.get_s(src)?.clone();
                    let val = regs.get(dst, index).map(|v| v)?;
                    let offset = regs.a16[offset as u8 as usize]?;
                    let end = offset.saturating_add(dst.layout().bytes() - 1);
                    s.adjust_len(end, true);
                    s.as_mut()[offset as usize..=end as usize].copy_from_slice(val.as_ref());
                    regs.s16.insert(src as u8, s);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(dst, index, MaybeNumber::none());
                })
            }
            BytesOp::Join(src1, src2, dst) => {
                let mut f = || -> Option<()> {
                    let (s1, s2) = regs.get_both_s(src1, src2)?;
                    let len = s1.len() + s2.len();
                    if len > u16::MAX as usize + 1 {
                        return None;
                    }
                    let mut d = s1.clone();
                    if len == u16::MAX as usize + 1 {
                        d.adjust_len((len - 1) as u16, true);
                    } else {
                        d.adjust_len(len as u16, false);
                    }
                    let mut d = ByteStr::with(s1);
                    d.as_mut()[s1.len()..].copy_from_slice(s2.as_ref());
                    regs.s16.insert(dst, d);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.s16.remove(&dst);
                })
            }
            BytesOp::Splt(flag, offset, src, dst1, dst2) => {
                todo!("#(6) complete bytestring opcode implementation")
            }
            BytesOp::Ins(flag, offset, src, dst) => {
                todo!("#(6) complete bytestring opcode implementation")
            }
            BytesOp::Del(flag, reg1, offset1, reg2, offset2, flag1, flag2, src, dst) => {
                todo!("#(6) complete bytestring opcode implementation")
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for DigestOp {
    #[inline]
    fn ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert("BPDIGEST");
        set
    }

    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep {
        let none;
        match self {
            DigestOp::Ripemd(src, dst) => {
                let s = regs.get_s(src);
                none = s.is_none();
                let hash = s.map(|s| ripemd160::Hash::hash(&s.bytes[..]).into_inner());
                regs.set(RegR::R160, dst, hash);
            }
            DigestOp::Sha256(src, dst) => {
                let s = regs.get_s(src);
                none = s.is_none();
                let hash = s.map(|s| sha256::Hash::hash(&s.bytes[..]).into_inner());
                regs.set(RegR::R256, dst, hash);
            }
            DigestOp::Sha512(src, dst) => {
                let s = regs.get_s(src);
                none = s.is_none();
                let hash = s.map(|s| sha512::Hash::hash(&s.bytes[..]).into_inner());
                regs.set(RegR::R512, dst, hash);
            }
        }
        if none {
            regs.st0 = false;
        }
        ExecStep::Next
    }
}

impl InstructionSet for Secp256k1Op {
    #[cfg(not(feature = "secp256k1"))]
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[cfg(feature = "secp256k1")]
    #[inline]
    fn ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert("SECP256K1");
        set
    }

    #[cfg(not(feature = "secp256k1"))]
    fn exec(self, _: &mut Registers, _: LibSite) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Secp256k1 instructions")
    }

    #[cfg(feature = "secp256k1")]
    fn exec(self, regs: &mut Registers, _site: LibSite) -> ExecStep {
        use secp256k1::{PublicKey, SecretKey};

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
    #[cfg(not(feature = "secp256k1"))]
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[cfg(feature = "secp256k1")]
    #[inline]
    fn ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert("ED25519");
        set
    }

    #[cfg(not(feature = "curve25519"))]
    fn exec(self, _: &mut Registers, _: LibSite) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Curve25519 instructions")
    }

    #[cfg(feature = "curve25519")]
    fn exec(self, _regs: &mut Registers, _site: LibSite) -> ExecStep {
        todo!("(#8) implement operations on Edwards curves")
    }
}

impl InstructionSet for NOp {
    #[inline]
    fn ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn exec(self, _: &mut Registers, _: LibSite) -> ExecStep { ExecStep::Next }
}
