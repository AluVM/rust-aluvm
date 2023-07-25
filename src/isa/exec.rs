// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2023 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023 UBIDECO Institute. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, BitXor, Neg, Rem, Shl, Shr};

use sha2::Digest;

use super::{
    ArithmeticOp, BitwiseOp, Bytecode, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp,
    Instr, MoveOp, PutOp, ReservedOp, Secp256k1Op,
};
use crate::data::{ByteArray, ByteStr, MaybeByteArray, MaybeNumber, Number, NumberLayout};
use crate::isa::{ExtendFlag, FloatEqFlag, IntFlags, MergeFlag, NoneEqFlag, SignFlag};
use crate::library::{constants, LibSite};
use crate::reg::{CoreRegs, NumericRegister, Reg32, RegA, RegA2, RegAR, RegR};

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
    /// Context: external data which are accessible to the ISA.
    type Context<'ctx>;

    /// ISA Extensions used by the provided instruction set.
    ///
    /// Each id must be up to 8 bytes and consist of upper case latin alphanumeric characters,
    /// starting with non-number.
    fn isa_ids() -> BTreeSet<&'static str>;

    /// ISA Extension IDs represented as a standard string (space-separated)
    ///
    /// Concatenated length of the ISA IDs joined via ' ' character must not exceed 128 bytes.
    #[inline]
    fn isa_string() -> String { Self::isa_ids().into_iter().collect::<Vec<_>>().join(" ") }

    /// ISA Extension IDs encoded in a standard way (space-separated)
    ///
    /// Concatenated length of the ISA IDs joined via ' ' character must not exceed 128 bytes.
    #[inline]
    fn isa_id() -> Box<[u8]> { Self::isa_string().as_bytes().into() }

    /// Checks whether provided ISA extension ID is supported by the current instruction set
    #[inline]
    fn is_supported(id: &str) -> bool { Self::isa_ids().contains(id) }

    /// Returns computational complexity of the instruction
    #[inline]
    fn complexity(&self) -> u64 { 1 }

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
    // TODO: Take the instruction by reference
    fn exec(&self, regs: &mut CoreRegs, site: LibSite, context: &Self::Context<'_>) -> ExecStep;
}

impl<Extension> InstructionSet for Instr<Extension>
where
    Extension: InstructionSet,
{
    type Context<'ctx> = Extension::Context<'ctx>;

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert(constants::ISA_ID_ALU);
        set.extend(DigestOp::isa_ids());
        set.extend(Secp256k1Op::isa_ids());
        set.extend(Curve25519Op::isa_ids());
        set
    }

    #[inline]
    fn exec(&self, regs: &mut CoreRegs, site: LibSite, ctx: &Self::Context<'_>) -> ExecStep {
        match self {
            Instr::ControlFlow(instr) => instr.exec(regs, site, &()),
            Instr::Put(instr) => instr.exec(regs, site, &()),
            Instr::Move(instr) => instr.exec(regs, site, &()),
            Instr::Cmp(instr) => instr.exec(regs, site, &()),
            Instr::Arithmetic(instr) => instr.exec(regs, site, &()),
            Instr::Bitwise(instr) => instr.exec(regs, site, &()),
            Instr::Bytes(instr) => instr.exec(regs, site, &()),
            Instr::Digest(instr) => instr.exec(regs, site, &()),
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.exec(regs, site, &()),
            #[cfg(feature = "curve25519")]
            Instr::Curve25519(instr) => instr.exec(regs, site, &()),
            Instr::ExtensionCodes(instr) => instr.exec(regs, site, ctx),
            Instr::ReservedInstruction(_) => ControlFlowOp::Fail.exec(regs, site, &()),
            Instr::Nop => ExecStep::Next,
        }
    }
}

impl InstructionSet for ControlFlowOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[inline]
    fn complexity(&self) -> u64 { 2 }

    fn exec(&self, regs: &mut CoreRegs, site: LibSite, _: &()) -> ExecStep {
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
                regs.jmp().map(|_| ExecStep::Jump(*offset)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Jif(offset) => {
                if regs.st0 {
                    regs.jmp().map(|_| ExecStep::Jump(*offset)).unwrap_or(ExecStep::Stop)
                } else {
                    ExecStep::Next
                }
            }
            ControlFlowOp::Routine(offset) => {
                regs.call(site).map(|_| ExecStep::Jump(*offset)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Call(site) => {
                regs.call(*site).map(|_| ExecStep::Call(*site)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Exec(site) => {
                regs.jmp().map(|_| ExecStep::Call(*site)).unwrap_or(ExecStep::Stop)
            }
            ControlFlowOp::Ret => regs.ret().map(ExecStep::Call).unwrap_or(ExecStep::Stop),
        }
    }
}

impl InstructionSet for PutOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[inline]
    fn complexity(&self) -> u64 { 2 }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        match self {
            PutOp::ClrA(reg, index) => {
                regs.set(reg, index, MaybeNumber::none());
            }
            PutOp::ClrF(reg, index) => {
                regs.set(reg, index, MaybeNumber::none());
            }
            PutOp::ClrR(reg, index) => {
                regs.set(reg, index, MaybeNumber::none());
            }
            PutOp::PutA(reg, index, number) => {
                if !regs.set(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutF(reg, index, number) => {
                if !regs.set(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutR(reg, index, number) => {
                if !regs.set(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutIfA(reg, index, number) => {
                if !regs.set_if(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutIfR(reg, index, number) => {
                if !regs.set_r_if(*reg, index, number.map(|n| n.into_byte_array(reg.bytes()).0)) {
                    regs.st0 = false;
                }
            }
        };
        ExecStep::Next
    }
}

impl InstructionSet for MoveOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
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
                regs.set_r(*reg, idx2, regs.get_r(*reg, idx1));
                regs.set_r(*reg, idx1, MaybeByteArray::none());
            }
            MoveOp::DupR(reg, idx1, idx2) => {
                regs.set_r(*reg, idx2, regs.get_r(*reg, idx1));
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
                let mut val = regs.get_r(*sreg, sidx);
                regs.st0 = val.reshape(dreg.bytes());
                regs.set_r(*dreg, didx, val);
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                let val1 = regs.get(sreg, sidx);
                let val2 = regs.get_r(*dreg, didx);
                let new_val1 = val2.map(|v| v.into_number(sreg.layout())).unwrap_or_default();
                let new_val2 = val1.map(|v| v.into_byte_array(dreg.bytes())).unwrap_or_default();
                regs.st0 = new_val1.1 && new_val2.1;
                regs.set(sreg, sidx, new_val1.0);
                regs.set_r(*dreg, didx, new_val2.0);
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
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        match self {
            CmpOp::GtA(sign_flag, reg, idx1, idx2) => {
                regs.st0 =
                    regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                        match bool::from(sign_flag) {
                            true => val1.into_signed().cmp(&val2.into_signed()),
                            false => val1.cmp(&val2),
                        }
                    }) == Some(Ordering::Greater);
            }
            CmpOp::GtF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    if *eq_flag == FloatEqFlag::Rounding {
                        val1.rounding_cmp(&val2)
                    } else {
                        val1.cmp(&val2)
                    }
                }) == Some(Ordering::Greater);
            }
            CmpOp::GtR(reg, idx1, idx2) => {
                regs.st0 =
                    regs.get_both_r(*reg, idx1, *reg, idx2).map(|(val1, val2)| val1.cmp(&val2))
                        == Some(Ordering::Greater);
            }
            CmpOp::LtA(sign_flag, reg, idx1, idx2) => {
                regs.st0 =
                    regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                        match bool::from(sign_flag) {
                            true => val1.into_signed().cmp(&val2.into_signed()),
                            false => val1.cmp(&val2),
                        }
                    }) == Some(Ordering::Less);
            }
            CmpOp::LtF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_both(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    if *eq_flag == FloatEqFlag::Rounding {
                        val1.rounding_cmp(&val2)
                    } else {
                        val1.cmp(&val2)
                    }
                }) == Some(Ordering::Less);
            }
            CmpOp::LtR(reg, idx1, idx2) => {
                regs.st0 =
                    regs.get_both_r(*reg, idx1, *reg, idx2).map(|(val1, val2)| val1.cmp(&val2))
                        == Some(Ordering::Less);
            }
            CmpOp::EqA(st, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_both(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| val1 == val2)
                    .unwrap_or(*st == NoneEqFlag::Equal);
            }
            CmpOp::EqF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_both(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| {
                        if *eq_flag == FloatEqFlag::Rounding {
                            val1.rounding_eq(&val2)
                        } else {
                            val1 == val2
                        }
                    })
                    .unwrap_or(false);
            }
            CmpOp::EqR(st, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_both_r(*reg, idx1, *reg, idx2)
                    .map(|(val1, val2)| val1 == val2)
                    .unwrap_or(*st == NoneEqFlag::Equal);
            }
            CmpOp::IfZA(reg, idx) => {
                regs.st0 = regs.get(reg, idx).map(Number::is_zero).unwrap_or(false)
            }
            CmpOp::IfZR(reg, idx) => {
                regs.st0 = regs.get_r(*reg, idx).map(ByteArray::is_zero).unwrap_or(false)
            }
            CmpOp::IfNA(reg, idx) => regs.st0 = regs.get(reg, idx).is_none(),
            CmpOp::IfNR(reg, idx) => regs.st0 = regs.get_r(*reg, idx).is_none(),
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
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[inline]
    fn complexity(&self) -> u64 {
        match self {
            ArithmeticOp::AddF(_, _, _, _)
            | ArithmeticOp::SubF(_, _, _, _)
            | ArithmeticOp::MulF(_, _, _, _)
            | ArithmeticOp::DivF(_, _, _, _) => 10,

            ArithmeticOp::AddA(_, _, _, _)
            | ArithmeticOp::SubA(_, _, _, _)
            | ArithmeticOp::MulA(_, _, _, _)
            | ArithmeticOp::DivA(_, _, _, _)
            | ArithmeticOp::Rem(_, _, _, _)
            | ArithmeticOp::Stp(_, _, _)
            | ArithmeticOp::Neg(_, _)
            | ArithmeticOp::Abs(_, _) => 1,
        }
    }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        let is_some = match self {
            ArithmeticOp::Abs(reg, idx) => {
                regs.set(reg, idx, regs.get(*reg, idx).and_then(Number::abs))
            }
            ArithmeticOp::AddA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_add(val2, *flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::AddF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_add(val2, *flags).into());
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::SubA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_sub(val2, *flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::SubF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_sub(val2, *flags).into());
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::MulA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_mul(val2, *flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::MulF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_mul(val2, *flags).into());
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::DivA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_div(val2, *flags));
                regs.set(reg, srcdst, res)
            }
            ArithmeticOp::DivF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_both(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_div(val2, *flags).into());
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
                    if step.as_i8() < 0 {
                        let mut n = Number::from(-step.as_i8());
                        debug_assert!(
                            n.reshape(val.layout()),
                            "reshape target byte length is always greater"
                        );
                        val.int_sub(n, IntFlags { signed: false, wrap: false })
                    } else {
                        let mut n = Number::from(*step);
                        debug_assert!(
                            n.reshape(val.layout()),
                            "reshape target byte length is always greater"
                        );
                        val.int_add(n, IntFlags { signed: false, wrap: false })
                    }
                }),
            ),
            ArithmeticOp::Neg(reg, idx) => {
                regs.set(reg, idx, regs.get(*reg, idx).and_then(Number::neg))
            }
        };
        regs.st0 = is_some;
        ExecStep::Next
    }
}

impl InstructionSet for BitwiseOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn exec(&self, regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        match self {
            BitwiseOp::And(reg, src1, src2, dst) => match reg {
                RegAR::A(a) => regs.op(a, src1, a, src2, a, dst, BitAnd::bitand),
                RegAR::R(r) => regs.op_r(*r, src1, *r, src2, *r, dst, BitAnd::bitand),
            },
            BitwiseOp::Or(reg, src1, src2, dst) => match reg {
                RegAR::A(a) => regs.op(a, src1, a, src2, a, dst, BitOr::bitor),
                RegAR::R(r) => regs.op_r(*r, src1, *r, src2, *r, dst, BitOr::bitor),
            },
            BitwiseOp::Xor(reg, src1, src2, dst) => match reg {
                RegAR::A(a) => regs.op(a, src1, a, src2, a, dst, BitXor::bitxor),
                RegAR::R(r) => regs.op_r(*r, src1, *r, src2, *r, dst, BitXor::bitxor),
            },
            BitwiseOp::Not(reg, idx) => {
                match reg {
                    RegAR::A(a) => regs.set(a, idx, !regs.get(a, idx)),
                    RegAR::R(r) => regs.set_r(*r, idx, !regs.get_r(*r, idx)),
                };
            }
            BitwiseOp::Shl(reg1, shift, reg2, srcdst) => match reg2 {
                RegAR::A(a) => {
                    let msb = regs.get(a, srcdst).unwrap_or_default()[a.bytes() - 1] & 0x80;
                    regs.st0 = msb == 0x80;
                    regs.op(a, srcdst, reg1, shift, a, srcdst, Shl::shl)
                }
                RegAR::R(r) => {
                    let shift = match reg1 {
                        RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as u16,
                        RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default(),
                    };
                    let res = regs.get_r(*r, srcdst).map(|bytes| {
                        let msb = bytes[..].last().copied().unwrap_or_default() & 0x80;
                        regs.st0 = msb == 0x80;
                        bytes.shl(shift)
                    });
                    regs.set_r(*r, srcdst, res);
                }
            },
            BitwiseOp::ShrA(flag, reg1, shift, reg2, srcdst) => {
                let res = regs.get_both(reg1, shift, reg2, srcdst).map(|(shift, val)| {
                    let lsb = val[0] & 1;
                    regs.st0 = lsb == 1;
                    if *flag == SignFlag::Signed {
                        val.into_signed().shr(shift)
                    } else {
                        val.shr(shift)
                    }
                });
                regs.set(reg2, srcdst, res);
            }
            BitwiseOp::ShrR(reg1, shift, reg2, srcdst) => {
                let shift = match reg1 {
                    RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as u16,
                    RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default(),
                };
                let res = regs.get_r(*reg2, srcdst).map(|bytes| {
                    let lsb = bytes[0] & 1;
                    regs.st0 = lsb == 1;
                    bytes.shr(shift)
                });
                regs.set_r(*reg2, srcdst, res);
            }
            BitwiseOp::Scl(reg1, shift, reg2, srcdst) => match reg2 {
                RegAR::A(a) => {
                    let msb = regs.get(a, srcdst).unwrap_or_default()[reg2.bytes() - 1] & 0x80;
                    regs.st0 = msb == 0x80;
                    regs.op(a, srcdst, reg1, shift, a, srcdst, Number::scl)
                }
                RegAR::R(r) => {
                    let shift = match reg1 {
                        RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as u16,
                        RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default(),
                    };
                    let shift = shift % reg2.bits();
                    let res = regs.get_r(*r, srcdst).map(|bytes| {
                        let msb = bytes[..].last().copied().unwrap_or_default() & 0x80;
                        regs.st0 = msb == 0x80;
                        bytes.scl(shift)
                    });
                    regs.set_r(*r, srcdst, res);
                }
            },
            BitwiseOp::Scr(reg1, shift, reg2, srcdst) => match reg2 {
                RegAR::A(a) => {
                    let lsb = regs.get(a, srcdst).unwrap_or_default()[0] & 1;
                    regs.st0 = lsb == 1;
                    regs.op(a, srcdst, reg1, shift, a, srcdst, Number::scr)
                }
                RegAR::R(r) => {
                    let shift = match reg1 {
                        RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as u16,
                        RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default(),
                    };
                    let shift = shift % reg2.bits();
                    let res = regs.get_r(*r, srcdst).map(|bytes| {
                        let lsb = bytes[0] & 1;
                        regs.st0 = lsb == 1;
                        bytes.scr(shift)
                    });
                    regs.set_r(*r, srcdst, res);
                }
            },
            BitwiseOp::RevA(reg, idx) => {
                regs.set(reg, idx, regs.get(reg, idx).map(Number::reverse_bits));
            }
            BitwiseOp::RevR(reg, idx) => {
                regs.set_r(*reg, idx, regs.get_r(*reg, idx).map(ByteArray::reverse_bits));
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for BytesOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[inline]
    fn complexity(&self) -> u64 { 5 }

    #[allow(warnings)]
    fn exec(&self, regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        match self {
            BytesOp::Put(reg, bytes, st0) => {
                regs.s16[reg.as_usize()] = Some(*bytes.clone());
                if *st0 {
                    regs.st0 = false
                }
            }
            BytesOp::Mov(reg1, reg2) => {
                let bs = regs.s16[reg1.as_usize()].clone();
                regs.s16[reg1.as_usize()] = None;
                regs.s16[reg2.as_usize()] = bs;
            }
            BytesOp::Swp(reg1, reg2) => {
                let bs1 = regs.s16[reg1.as_usize()].clone();
                let bs2 = regs.s16[reg2.as_usize()].clone();
                regs.s16[reg1.as_usize()] = bs2;
                regs.s16[reg2.as_usize()] = bs1;
            }
            BytesOp::Fill(reg, offset1, offset2, value, flag) => {
                let mut f = || -> Option<()> {
                    let o1 = regs.a16[offset1.to_usize()]?;
                    let o2 = regs.a16[offset2.to_usize()]?;
                    let range = o1..o2;
                    let val = regs.a8[value.to_usize()]?;
                    let ref mut bs = regs.s16[reg.as_usize()];
                    let bs = if let Some(s) = bs {
                        s
                    } else {
                        *bs = Some(ByteStr::default());
                        bs.as_mut().expect("rust optionals are broken")
                    };
                    if bs.len() <= range.end && *flag == ExtendFlag::Fail {
                        return None;
                    }
                    bs.fill(range, val);
                    Some(())
                };
                f().unwrap_or_else(|| regs.st0 = false);
            }
            BytesOp::Len(src, reg, dst) => {
                let mut f = || -> Option<()> {
                    let s = regs.get_s(*src)?;
                    let len = s.len();
                    if !reg.int_layout().fits_usize(len as usize) {
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
                    let val = regs.a8[*byte as u8 as usize]?;
                    let bs = regs.s16[src.as_usize()].as_ref()?;
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
                let s1 = regs.get_s(*reg1);
                let s2 = regs.get_s(*reg2);
                regs.st0 = match (s1, s2) {
                    (Some(s1), Some(s2)) => s1 == s2,
                    (None, None) => true,
                    _ => false,
                };
            }
            BytesOp::Find(reg1, reg2) => {
                let mut f = || -> Option<()> {
                    let (s1, s2) = regs.get_both_s(*reg1, *reg2)?;
                    let r1 = s1.as_ref();
                    let r2 = s2.as_ref();
                    let count = r1.windows(r2.len()).filter(|r1| *r1 == r2).count();
                    assert!(count <= u16::MAX as usize);
                    regs.set(RegA::A16, Reg32::Reg0, count as u16);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(RegA::A16, Reg32::Reg0, MaybeNumber::none());
                })
            }
            BytesOp::Rev(reg1, reg2) => {
                let mut f = || -> Option<()> {
                    let mut s = regs.get_s(*reg1)?.clone();
                    let bs = s.as_mut();
                    bs.reverse();
                    regs.s16[reg2.as_usize()] = Some(s);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.s16[reg2.as_usize()] = None;
                })
            }
            BytesOp::Con(reg1, reg2, n, offset_dst, len_dst) => {
                let mut f = || -> Option<()> {
                    let (s1, s2) = (regs.get_s(*reg1)?, regs.get_s(*reg2)?);
                    let (r1, r2) = (s1.as_ref(), s2.as_ref());
                    let n = regs.a16[*n as u8 as usize]?;
                    let size = ::core::cmp::min(s1.len(), s2.len());
                    let mut elems = (0..)
                        .zip(r1.iter().zip(r2).map(|(c1, c2)| c1 == c2))
                        .take(size as usize)
                        .skip_while(|(_, c)| !*c);
                    for _ in 0..n {
                        while let Some((_, false)) = elems.next() {}
                        while let Some((_, true)) = elems.next() {}
                    }
                    let begin = elems.next();
                    let end = elems.skip_while(|(_, c)| *c).next();
                    let (offset, len) = match (begin, end) {
                        (Some((b, _)), Some((e, _))) => (b, e - b),
                        (Some((b, _)), None) => (b, size - b),
                        _ => return None,
                    };
                    regs.set(RegA::A16, offset_dst, offset);
                    regs.set(RegA::A16, len_dst, len);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(RegA::A16, offset_dst, MaybeNumber::none());
                    regs.set(RegA::A16, len_dst, MaybeNumber::none());
                })
            }
            BytesOp::Extr(src, dst, index, offset) => {
                let mut f = || -> Option<()> {
                    let s_len = regs.get_s(*src)?.len();
                    let offset = regs.a16[*offset as u8 as usize].filter(|e| *e < s_len)?;
                    let end = offset
                        .checked_add(dst.layout().bytes())
                        .filter(|e| *e < s_len)
                        .unwrap_or_else(|| {
                            regs.st0 = false;
                            s_len
                        });
                    let num = Number::from_slice(
                        &regs.get_s(*src)?.as_ref()[offset as usize..end as usize],
                    );
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
                    let mut s = regs.get_s(*src)?.clone();
                    let val = regs.get_r(*dst, index).map(|v| v)?;
                    let offset = regs.a16[*offset as u8 as usize]?;
                    let end = offset.saturating_add(dst.layout().bytes() - 1);
                    s.adjust_len(end);
                    s.as_mut()[offset as usize..=end as usize].copy_from_slice(val.as_ref());
                    regs.s16[src.as_usize()] = Some(s);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set(dst, index, MaybeNumber::none());
                })
            }
            BytesOp::Join(src1, src2, dst) => {
                let mut f = || -> Option<()> {
                    let (s1, s2) = regs.get_both_s(*src1, *src2)?;
                    if s1.len() as usize + s2.len() as usize > u16::MAX as usize {
                        return None;
                    }
                    let len = s1.len() + s2.len();
                    let mut d = s1.clone();
                    d.adjust_len(len);
                    let mut d = ByteStr::with(s1);
                    d.as_mut()[s1.len() as usize..].copy_from_slice(s2.as_ref());
                    regs.s16[dst.as_usize()] = Some(d);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.s16[dst.as_usize()] = None;
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
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert(constants::ISA_ID_BPDIGEST);
        set
    }

    #[inline]
    fn complexity(&self) -> u64 { 100 }

    fn exec(&self, regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        let none;
        match self {
            DigestOp::Ripemd(src, dst) => {
                let s = regs.get_s(*src);
                none = s.is_none();
                let hash = s.map(|s| {
                    let mut hash: [u8; 20] = ripemd::Ripemd160::digest(s.as_ref()).into();
                    // RIPEMD-160 is big-endian
                    hash.reverse();
                    hash
                });
                regs.set(RegR::R160, dst, hash);
            }
            DigestOp::Sha256(src, dst) => {
                let s = regs.get_s(*src);
                none = s.is_none();
                let hash: Option<[u8; 32]> = s.map(|s| sha2::Sha256::digest(s.as_ref()).into());
                regs.set(RegR::R256, dst, hash);
            }
            DigestOp::Sha512(src, dst) => {
                let s = regs.get_s(*src);
                none = s.is_none();
                let hash: Option<[u8; 64]> = s.map(|s| sha2::Sha512::digest(s.as_ref()).into());
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
    type Context<'ctx> = ();

    #[cfg(not(feature = "secp256k1"))]
    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[cfg(feature = "secp256k1")]
    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert(constants::ISA_ID_SECP256K);
        set
    }

    #[inline]
    fn complexity(&self) -> u64 { 1000 }

    #[cfg(not(feature = "secp256k1"))]
    fn exec(&self, _: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Secp256k1 instructions")
    }

    #[cfg(feature = "secp256k1")]
    fn exec(&self, regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        use secp256k1::{PublicKey, SecretKey, SECP256K1};

        match self {
            Secp256k1Op::Gen(src, dst) => {
                let res = regs
                    .get_r(RegR::R256, src)
                    .and_then(|mut src| {
                        let src = src.as_mut();
                        // little endian to big endian
                        src.reverse();
                        SecretKey::from_slice(src).ok()
                    })
                    .map(|sk| PublicKey::from_secret_key(SECP256K1, &sk))
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| ByteArray::from_slice(&pk[1..]));
                regs.set_r(RegR::R512, dst, res);
            }

            Secp256k1Op::Mul(block, scal, src, dst) => {
                let reg = block.into_reg(256).expect("register set does not match standard");
                let res = match reg {
                    RegAR::A(a) => {
                        MaybeByteArray::from(regs.get(a, scal).map(|n| n.into_byte_array(256).0))
                    }
                    RegAR::R(r) => regs.get_r(r, scal),
                }
                .and_then(|scal| {
                    regs.get_r(RegR::R512, src)
                        .and_then(|val| {
                            let mut pk = [4u8; 65];
                            pk[1..].copy_from_slice(val.as_ref());
                            PublicKey::from_slice(&pk).ok()
                        })
                        .map(|pk| (scal, pk))
                })
                .and_then(|(scal, pk)| {
                    let mut buf = [0u8; 32];
                    buf.copy_from_slice(scal.as_ref());
                    let scal = secp256k1::Scalar::from_le_bytes(buf).ok()?;
                    pk.mul_tweak(SECP256K1, &scal).ok()
                })
                .as_ref()
                .map(PublicKey::serialize_uncompressed)
                .map(|pk| ByteArray::from_slice(&pk[1..]));
                regs.set_r(RegR::R512, dst, res);
            }

            Secp256k1Op::Add(src, srcdst) => {
                let res = regs
                    .get_r(RegR::R512, src)
                    .and_then(|val| {
                        let mut pk1 = [4u8; 65];
                        pk1[1..].copy_from_slice(val.as_ref());
                        PublicKey::from_slice(&pk1).ok()
                    })
                    .and_then(|pk1| {
                        regs.get_r(RegR::R512, srcdst).and_then(|val| {
                            let mut pk2 = [4u8; 65];
                            pk2[1..].copy_from_slice(val.as_ref());
                            PublicKey::from_slice(&pk2).ok().map(|pk2| (pk1, pk2))
                        })
                    })
                    .and_then(|(pk1, pk2)| pk1.combine(&pk2).ok())
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set(RegR::R512, srcdst, res);
            }

            Secp256k1Op::Neg(src, dst) => {
                let res = regs
                    .get_r(RegR::R512, src)
                    .and_then(|val| {
                        let mut pk = [4u8; 65];
                        pk[1..].copy_from_slice(&val[..]);
                        PublicKey::from_slice(&pk).ok()
                    })
                    .map(|pk| pk.negate(SECP256K1))
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
    type Context<'ctx> = ();

    #[cfg(not(feature = "curve25519"))]
    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    #[cfg(feature = "curve25519")]
    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        set.insert(constants::ISA_ID_ED25519);
        set
    }

    #[inline]
    fn complexity(&self) -> u64 { 1000 }

    #[cfg(not(feature = "curve25519"))]
    fn exec(&self, _: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Curve25519 instructions")
    }

    #[cfg(feature = "curve25519")]
    fn exec(&self, regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        use amplify::num::u256;
        use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
        use curve25519_dalek::scalar::Scalar;

        let get_scalar = |src: ByteArray| {
            let mut scal = [0u8; 32];
            scal.copy_from_slice(&src.as_ref()[..32]);
            Scalar::from_bits(scal)
        };

        let from_scalar = |scal: Scalar| {
            let mut n = [0u8; 64];
            n[..32].copy_from_slice(scal.as_bytes());
            n[32..].copy_from_slice((ED25519_BASEPOINT_POINT * scal).compress().as_bytes());
            ByteArray::from_slice(n)
        };

        match self {
            Curve25519Op::Gen(src, dst) => {
                let res = regs.get_r(RegR::R256, src).map(get_scalar).map(from_scalar);
                regs.set_r(RegR::R512, dst, res);
            }
            Curve25519Op::Mul(block, scal, src, dst) => {
                let reg = block.into_reg(256).expect("register set does not match standard");
                let lhs = match reg {
                    RegAR::A(a) => {
                        regs.get(a, scal).map(|n| n.into_byte_array(256).0).map(get_scalar)
                    }
                    RegAR::R(r) => regs.get_r(r, scal).map(get_scalar),
                };
                let rhs = match reg {
                    RegAR::A(a) => {
                        regs.get(a, src).map(|n| n.into_byte_array(256).0).map(get_scalar)
                    }
                    RegAR::R(r) => regs.get_r(r, src).map(get_scalar),
                };
                let res = lhs.zip(rhs).map(|(lhs, rhs)| lhs * rhs).map(from_scalar);
                regs.set_r(RegR::R512, dst, res);
            }
            Curve25519Op::Add(lhs, rhs, dst, overflow) => {
                let lhs = regs
                    .get_r(RegR::R512, lhs)
                    .map(get_scalar)
                    .map(|s| u256::from_le_bytes(s.to_bytes()));
                let rhs = regs
                    .get_r(RegR::R512, rhs)
                    .map(get_scalar)
                    .map(|s| u256::from_le_bytes(s.to_bytes()));
                let res = lhs
                    .zip(rhs)
                    .and_then(|(lhs, rhs)| {
                        let scal = Scalar::from_bits((lhs + rhs).to_le_bytes());
                        match !*overflow && !scal.is_canonical() {
                            true => {
                                regs.st0 = false;
                                None
                            }
                            false => Some(scal.reduce()),
                        }
                    })
                    .map(from_scalar);
                regs.set_r(RegR::R512, dst, res);
            }
            Curve25519Op::Neg(src, dst) => {
                let res = regs.get_r(RegR::R512, src).map(get_scalar).map(|s| -s).map(from_scalar);
                regs.set_r(RegR::R512, dst, res);
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for ReservedOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn exec(&self, regs: &mut CoreRegs, site: LibSite, ctx: &()) -> ExecStep {
        ControlFlowOp::Fail.exec(regs, site, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "secp256k1")]
    use crate::reg::{Reg8, RegBlockAR};

    #[test]
    fn bytes_con_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        let s1 = "apple_banana_kiwi".as_bytes();
        let s2 = "apple@banana@kiwi".as_bytes();
        BytesOp::Put(1.into(), Box::new(ByteStr::with(s1)), false).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Put(2.into(), Box::new(ByteStr::with(s2)), false).exec(
            &mut register,
            lib_site,
            &(),
        );
        // apple (0th fragment)
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(0).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1).unwrap(), Number::from(0u16));
        assert_eq!(register.get(RegA::A16, Reg32::Reg2).unwrap(), Number::from(5u16));
        assert!(register.st0);
        // banana (1st fragment)
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(1).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1).unwrap(), Number::from(6u16));
        assert_eq!(register.get(RegA::A16, Reg32::Reg2).unwrap(), Number::from(6u16));
        assert!(register.st0);
        // kiwi (2nd fragment)
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(2).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1).unwrap(), Number::from(13u16));
        assert_eq!(register.get(RegA::A16, Reg32::Reg2).unwrap(), Number::from(4u16));
        assert!(register.st0);
        // no 3rd fragment
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(3).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1), MaybeNumber::none());
        assert_eq!(register.get(RegA::A16, Reg32::Reg2), MaybeNumber::none());
        assert!(!register.st0);

        let s1 = "aaa".as_bytes();
        let s2 = "bbb".as_bytes();
        BytesOp::Put(1.into(), Box::new(ByteStr::with(s1)), false).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Put(2.into(), Box::new(ByteStr::with(s2)), false).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(0).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1), MaybeNumber::none());
        assert_eq!(register.get(RegA::A16, Reg32::Reg2), MaybeNumber::none());
        assert!(!register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());

        let s1 = [0u8; u16::MAX as usize];
        let s2 = [0u8; u16::MAX as usize];
        BytesOp::Put(1.into(), Box::new(ByteStr::with(s1)), false).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Put(2.into(), Box::new(ByteStr::with(s2)), false).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(0).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1).unwrap(), Number::from(0u16));
        assert_eq!(register.get(RegA::A16, Reg32::Reg2).unwrap(), Number::from(u16::MAX));
        assert!(register.st0);
        PutOp::PutA(RegA::A16, Reg32::Reg0, MaybeNumber::from(1).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        BytesOp::Con(1.into(), 2.into(), Reg32::Reg0, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(register.get(RegA::A16, Reg32::Reg1), MaybeNumber::none());
        assert_eq!(register.get(RegA::A16, Reg32::Reg2), MaybeNumber::none());
        assert!(!register.st0);
    }

    #[test]
    #[cfg(feature = "secp256k1")]
    fn secp256k1_add_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        PutOp::PutR(RegR::R256, Reg32::Reg0, MaybeNumber::from(600u16).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg1, MaybeNumber::from(1200u16).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg2, MaybeNumber::from(1800u16).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Secp256k1Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Secp256k1Op::Gen(Reg32::Reg1, Reg8::Reg1).exec(&mut register, lib_site, &());
        Secp256k1Op::Add(Reg32::Reg0, Reg8::Reg1).exec(&mut register, lib_site, &());
        Secp256k1Op::Gen(Reg32::Reg2, Reg8::Reg2).exec(&mut register, lib_site, &());
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
    }

    #[test]
    #[cfg(feature = "secp256k1")]
    fn secp256k1_mul_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        PutOp::PutR(RegR::R256, Reg32::Reg0, MaybeNumber::from(2u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg1, MaybeNumber::from(3u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg2, MaybeNumber::from(6u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Secp256k1Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Secp256k1Op::Mul(RegBlockAR::R, Reg32::Reg1, Reg32::Reg0, Reg32::Reg1).exec(
            &mut register,
            lib_site,
            &(),
        );
        Secp256k1Op::Gen(Reg32::Reg2, Reg8::Reg2).exec(&mut register, lib_site, &());
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
    }

    #[test]
    #[cfg(feature = "secp256k1")]
    fn secp256k1_neg_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        PutOp::PutR(RegR::R256, Reg32::Reg0, MaybeNumber::from(1u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Secp256k1Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Secp256k1Op::Neg(Reg32::Reg0, Reg8::Reg1).exec(&mut register, lib_site, &());
        Secp256k1Op::Neg(Reg32::Reg1, Reg8::Reg2).exec(&mut register, lib_site, &());
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg0, Reg32::Reg1).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(false, register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());
        assert_eq!(true, register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg0, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
        PutOp::PutR(RegR::R256, Reg32::Reg4, MaybeNumber::from(5u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg5, MaybeNumber::from(6u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Secp256k1Op::Gen(Reg32::Reg4, Reg8::Reg4).exec(&mut register, lib_site, &());
        Secp256k1Op::Gen(Reg32::Reg5, Reg8::Reg5).exec(&mut register, lib_site, &());
        // -G + 6G
        Secp256k1Op::Add(Reg32::Reg1, Reg8::Reg5).exec(&mut register, lib_site, &());
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg4, Reg32::Reg5).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
    }

    #[test]
    #[cfg(feature = "curve25519")]
    fn curve25519_mul_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        PutOp::PutR(RegR::R256, Reg32::Reg0, MaybeNumber::from(2u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg1, MaybeNumber::from(3u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg2, MaybeNumber::from(6u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Curve25519Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Curve25519Op::Mul(RegBlockAR::R, Reg32::Reg1, Reg32::Reg0, Reg32::Reg1).exec(
            &mut register,
            lib_site,
            &(),
        );
        Curve25519Op::Gen(Reg32::Reg2, Reg8::Reg2).exec(&mut register, lib_site, &());
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg0, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(false, register.st0);
    }

    #[test]
    #[cfg(feature = "curve25519")]
    fn curve25519_add_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        PutOp::PutR(RegR::R256, Reg32::Reg0, MaybeNumber::from(600u16).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg1, MaybeNumber::from(1200u16).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg2, MaybeNumber::from(1800u16).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Curve25519Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg1, Reg8::Reg1).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg2, Reg8::Reg2).exec(&mut register, lib_site, &());
        Curve25519Op::Add(Reg32::Reg0, Reg32::Reg1, Reg32::Reg3, false).exec(
            &mut register,
            lib_site,
            &(),
        );
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg2, Reg32::Reg3).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
    }

    #[test]
    #[cfg(feature = "curve25519")]
    fn curve25519_add_overflow_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        let l_plus_two_bytes: [u8; 32] = [
            0xef, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9,
            0xde, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x10,
        ];
        PutOp::PutR(
            RegR::R256,
            Reg32::Reg0,
            MaybeNumber::from(Number::from_slice(l_plus_two_bytes)).into(),
        )
        .exec(&mut register, lib_site, &());
        PutOp::PutR(RegR::R256, Reg32::Reg1, MaybeNumber::from(1u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg2, MaybeNumber::from(3u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Curve25519Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg1, Reg8::Reg1).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg2, Reg8::Reg2).exec(&mut register, lib_site, &());
        Curve25519Op::Add(Reg32::Reg0, Reg32::Reg1, Reg32::Reg3, false).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(false, register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());
        Curve25519Op::Add(Reg32::Reg0, Reg32::Reg1, Reg32::Reg3, true).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg2, Reg32::Reg3).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
    }

    #[test]
    #[cfg(feature = "curve25519")]
    fn curve25519_neg_test() {
        let mut register = CoreRegs::default();
        let lib_site = LibSite::default();
        PutOp::PutR(RegR::R256, Reg32::Reg0, MaybeNumber::from(1u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Curve25519Op::Gen(Reg32::Reg0, Reg8::Reg0).exec(&mut register, lib_site, &());
        Curve25519Op::Neg(Reg32::Reg0, Reg8::Reg1).exec(&mut register, lib_site, &());
        Curve25519Op::Neg(Reg32::Reg1, Reg8::Reg2).exec(&mut register, lib_site, &());
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg0, Reg32::Reg1).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(false, register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());
        assert_eq!(true, register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg0, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
        PutOp::PutR(RegR::R256, Reg32::Reg4, MaybeNumber::from(5u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        PutOp::PutR(RegR::R256, Reg32::Reg5, MaybeNumber::from(6u8).into()).exec(
            &mut register,
            lib_site,
            &(),
        );
        Curve25519Op::Gen(Reg32::Reg4, Reg8::Reg4).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg5, Reg8::Reg5).exec(&mut register, lib_site, &());
        // -G + 6G
        Curve25519Op::Add(Reg32::Reg1, Reg32::Reg5, Reg32::Reg6, true).exec(
            &mut register,
            lib_site,
            &(),
        );
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg4, Reg32::Reg6).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert_eq!(true, register.st0);
    }
}
