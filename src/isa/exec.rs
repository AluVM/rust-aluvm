// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Institute. All rights reserved.
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

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, BitXor, Neg, Rem, Shl, Shr};

use sha2::Digest;

use super::{
    ArithmeticOp, BitwiseOp, Bytecode, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp,
    Instr, MoveOp, PutOp, ReservedOp, Secp256k1Op,
};
use crate::data::{ByteStr, MaybeNumber, Number, NumberLayout};
use crate::isa::{ExtendFlag, FloatEqFlag, IntFlags, MergeFlag, NoneEqFlag, SignFlag};
use crate::library::{constants, LibSite};
use crate::reg::{CoreRegs, NumericRegister, Reg, Reg32, RegA, RegA2, RegAR, RegBlockAR, RegR};

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

    /// Lists all registers which are used by the instruction.
    fn regs(&self) -> BTreeSet<Reg> {
        let mut regs = self.src_regs();
        regs.extend(self.dst_regs());
        regs
    }

    /// List of registers which value is taken into the account by the instruction.
    fn src_regs(&self) -> BTreeSet<Reg>;

    /// List of registers which value may be changed by the instruction.
    fn dst_regs(&self) -> BTreeSet<Reg>;

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
        set.extend(Extension::isa_ids());
        set
    }

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            Instr::ControlFlow(instr) => instr.src_regs(),
            Instr::Put(instr) => instr.src_regs(),
            Instr::Move(instr) => instr.src_regs(),
            Instr::Cmp(instr) => instr.src_regs(),
            Instr::Arithmetic(instr) => instr.src_regs(),
            Instr::Bitwise(instr) => instr.src_regs(),
            Instr::Bytes(instr) => instr.src_regs(),
            Instr::Digest(instr) => instr.src_regs(),
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.src_regs(),
            #[cfg(feature = "curve25519")]
            Instr::Curve25519(instr) => instr.src_regs(),
            Instr::ExtensionCodes(instr) => instr.src_regs(),
            Instr::ReservedInstruction(instr) => instr.src_regs(),
            Instr::Nop => bset![],
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            Instr::ControlFlow(instr) => instr.dst_regs(),
            Instr::Put(instr) => instr.dst_regs(),
            Instr::Move(instr) => instr.dst_regs(),
            Instr::Cmp(instr) => instr.dst_regs(),
            Instr::Arithmetic(instr) => instr.dst_regs(),
            Instr::Bitwise(instr) => instr.dst_regs(),
            Instr::Bytes(instr) => instr.dst_regs(),
            Instr::Digest(instr) => instr.dst_regs(),
            #[cfg(feature = "secp256k1")]
            Instr::Secp256k1(instr) => instr.dst_regs(),
            #[cfg(feature = "curve25519")]
            Instr::Curve25519(instr) => instr.dst_regs(),
            Instr::ExtensionCodes(instr) => instr.dst_regs(),
            Instr::ReservedInstruction(instr) => instr.dst_regs(),
            Instr::Nop => bset![],
        }
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

    fn src_regs(&self) -> BTreeSet<Reg> { bset![] }

    fn dst_regs(&self) -> BTreeSet<Reg> { bset![] }

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

    fn src_regs(&self) -> BTreeSet<Reg> { bset![] }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            PutOp::ClrA(_, _) | PutOp::ClrF(_, _) | PutOp::ClrR(_, _) => bset![],
            PutOp::PutA(reg, reg32, _) => bset![Reg::A(*reg, *reg32)],
            PutOp::PutF(reg, reg32, _) => bset![Reg::F(*reg, *reg32)],
            PutOp::PutR(reg, reg32, _) => bset![Reg::R(*reg, *reg32)],
            PutOp::PutIfA(reg, reg32, _) => bset![Reg::A(*reg, *reg32)],
            PutOp::PutIfR(reg, reg32, _) => bset![Reg::R(*reg, *reg32)],
        }
    }

    #[inline]
    fn complexity(&self) -> u64 { 2 }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        match self {
            PutOp::ClrA(reg, index) => {
                regs.set_n(reg, index, MaybeNumber::none());
            }
            PutOp::ClrF(reg, index) => {
                regs.set_n(reg, index, MaybeNumber::none());
            }
            PutOp::ClrR(reg, index) => {
                regs.set_n(reg, index, MaybeNumber::none());
            }
            PutOp::PutA(reg, index, number) => {
                if !regs.set_n(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutF(reg, index, number) => {
                if !regs.set_n(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutR(reg, index, number) => {
                if !regs.set_n(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutIfA(reg, index, number) => {
                if !regs.set_n_if(reg, index, **number) {
                    regs.st0 = false;
                }
            }
            PutOp::PutIfR(reg, index, number) => {
                if !regs.set_n_if(reg, index, **number) {
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

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            MoveOp::MovA(reg, idx1, _idx2) => {
                bset![Reg::A(*reg, *idx1)]
            }
            MoveOp::DupA(reg, idx1, _idx2) => {
                bset![Reg::A(*reg, *idx1)]
            }
            MoveOp::SwpA(reg, idx1, idx2) => {
                bset![Reg::A(*reg, *idx1), Reg::A(*reg, *idx2)]
            }
            MoveOp::MovF(reg, idx1, _idx2) => {
                bset![Reg::F(*reg, *idx1)]
            }
            MoveOp::DupF(reg, idx1, _idx2) => {
                bset![Reg::F(*reg, *idx1)]
            }
            MoveOp::SwpF(reg, idx1, idx2) => {
                bset![Reg::F(*reg, *idx1), Reg::F(*reg, *idx2)]
            }
            MoveOp::MovR(reg, idx1, _idx2) => {
                bset![Reg::R(*reg, *idx1)]
            }
            MoveOp::DupR(reg, idx1, _idx2) => {
                bset![Reg::R(*reg, *idx1)]
            }

            MoveOp::CpyA(sreg, sidx, _dreg, _didx) => {
                bset![Reg::A(*sreg, *sidx)]
            }
            MoveOp::CnvA(sreg, sidx, _dreg, _didx) => {
                bset![Reg::A(*sreg, *sidx)]
            }
            MoveOp::CnvF(sreg, sidx, _dreg, _didx) => {
                bset![Reg::F(*sreg, *sidx)]
            }
            MoveOp::CpyR(sreg, sidx, _dreg, _didx) => {
                bset![Reg::R(*sreg, *sidx)]
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                bset![Reg::A(*sreg, *sidx), Reg::R(*dreg, *didx)]
            }
            MoveOp::CnvAF(sreg, sidx, _dreg, _didx) => {
                bset![Reg::A(*sreg, *sidx)]
            }
            MoveOp::CnvFA(sreg, sidx, _dreg, _didx) => {
                bset![Reg::F(*sreg, *sidx)]
            }
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            MoveOp::MovA(reg, idx1, idx2) => {
                bset![Reg::A(*reg, *idx1), Reg::A(*reg, *idx2)]
            }
            MoveOp::DupA(reg, _idx1, idx2) => {
                bset![Reg::A(*reg, *idx2)]
            }
            MoveOp::SwpA(reg, idx1, idx2) => {
                bset![Reg::A(*reg, *idx1), Reg::A(*reg, *idx2)]
            }
            MoveOp::MovF(reg, idx1, idx2) => {
                bset![Reg::F(*reg, *idx1), Reg::F(*reg, *idx2)]
            }
            MoveOp::DupF(reg, _idx1, idx2) => {
                bset![Reg::F(*reg, *idx2)]
            }
            MoveOp::SwpF(reg, idx1, idx2) => {
                bset![Reg::F(*reg, *idx1), Reg::F(*reg, *idx2)]
            }
            MoveOp::MovR(reg, idx1, idx2) => {
                bset![Reg::R(*reg, *idx1), Reg::R(*reg, *idx2)]
            }
            MoveOp::DupR(reg, _idx1, idx2) => {
                bset![Reg::R(*reg, *idx2)]
            }

            MoveOp::CpyA(_sreg, _sidx, dreg, didx) => {
                bset![Reg::A(*dreg, *didx)]
            }
            MoveOp::CnvA(_sreg, _sidx, dreg, didx) => {
                bset![Reg::A(*dreg, *didx)]
            }
            MoveOp::CnvF(_sreg, _sidx, dreg, didx) => {
                bset![Reg::F(*dreg, *didx)]
            }
            MoveOp::CpyR(_sreg, _sidx, dreg, didx) => {
                bset![Reg::R(*dreg, *didx)]
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                bset![Reg::A(*sreg, *sidx), Reg::R(*dreg, *didx)]
            }
            MoveOp::CnvAF(_sreg, _sidx, dreg, didx) => {
                bset![Reg::F(*dreg, *didx)]
            }
            MoveOp::CnvFA(_sreg, _sidx, dreg, didx) => {
                bset![Reg::A(*dreg, *didx)]
            }
        }
    }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        match self {
            MoveOp::MovA(reg, idx1, idx2) => {
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
                regs.set_n(reg, idx1, MaybeNumber::none());
            }
            MoveOp::DupA(reg, idx1, idx2) => {
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
            }
            MoveOp::SwpA(reg, idx1, idx2) => {
                let val = regs.get_n(reg, idx2);
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
                regs.set_n(reg, idx1, val);
            }
            MoveOp::MovF(reg, idx1, idx2) => {
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
                regs.set_n(reg, idx1, MaybeNumber::none());
            }
            MoveOp::DupF(reg, idx1, idx2) => {
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
            }
            MoveOp::SwpF(reg, idx1, idx2) => {
                let val = regs.get_n(reg, idx2);
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
                regs.set_n(reg, idx1, val);
            }
            MoveOp::MovR(reg, idx1, idx2) => {
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
                regs.set_n(reg, idx1, MaybeNumber::none());
            }
            MoveOp::DupR(reg, idx1, idx2) => {
                regs.set_n(reg, idx2, regs.get_n(reg, idx1));
            }

            MoveOp::CpyA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get_n(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set_n(dreg, didx, val);
            }
            MoveOp::CnvA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get_n(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout().into_signed());
                regs.set_n(dreg, didx, val);
            }
            MoveOp::CnvF(sreg, sidx, dreg, didx) => {
                let mut val = regs.get_n(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set_n(dreg, didx, val);
            }
            MoveOp::CpyR(sreg, sidx, dreg, didx) => {
                let mut val = regs.get_n(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set_n(dreg, didx, val);
            }
            MoveOp::SpyAR(sreg, sidx, dreg, didx) => {
                let mut val1 = regs.get_n(sreg, sidx);
                let mut val2 = regs.get_n(dreg, didx);
                regs.st0 = val1.reshape(dreg.layout()) && val2.reshape(sreg.layout());
                regs.set_n(dreg, didx, val1);
                regs.set_n(sreg, sidx, val2);
            }
            MoveOp::CnvAF(sreg, sidx, dreg, didx) => {
                let mut val = regs.get_n(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set_n(dreg, didx, val);
            }
            MoveOp::CnvFA(sreg, sidx, dreg, didx) => {
                let mut val = regs.get_n(sreg, sidx);
                regs.st0 = val.reshape(dreg.layout());
                regs.set_n(dreg, didx, val);
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for CmpOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            CmpOp::GtA(_, reg, idx1, idx2) => {
                bset![Reg::A(*reg, *idx1), Reg::A(*reg, *idx2)]
            }
            CmpOp::LtA(_, reg, idx1, idx2) => {
                bset![Reg::A(*reg, *idx1), Reg::A(*reg, *idx2)]
            }
            CmpOp::GtF(_, reg, idx1, idx2) => {
                bset![Reg::F(*reg, *idx1), Reg::F(*reg, *idx2)]
            }
            CmpOp::LtF(_, reg, idx1, idx2) => {
                bset![Reg::F(*reg, *idx1), Reg::F(*reg, *idx2)]
            }
            CmpOp::GtR(reg, idx1, idx2) => {
                bset![Reg::R(*reg, *idx1), Reg::R(*reg, *idx2)]
            }
            CmpOp::LtR(reg, idx1, idx2) => {
                bset![Reg::R(*reg, *idx1), Reg::R(*reg, *idx2)]
            }
            CmpOp::EqA(_, reg, idx1, idx2) => {
                bset![Reg::A(*reg, *idx1), Reg::A(*reg, *idx2)]
            }
            CmpOp::EqF(_, reg, idx1, idx2) => {
                bset![Reg::F(*reg, *idx1), Reg::F(*reg, *idx2)]
            }
            CmpOp::EqR(_, reg, idx1, idx2) => {
                bset![Reg::R(*reg, *idx1), Reg::R(*reg, *idx2)]
            }

            CmpOp::IfZA(reg, idx) | CmpOp::IfNA(reg, idx) => {
                bset![Reg::A(*reg, *idx)]
            }
            CmpOp::IfZR(reg, idx) | CmpOp::IfNR(reg, idx) => {
                bset![Reg::R(*reg, *idx)]
            }
            CmpOp::St(_, _, _) => {
                bset![]
            }
            CmpOp::StInv => bset![],
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            CmpOp::St(_, reg, idx) => {
                bset![Reg::A(*reg, (*idx).into())]
            }
            _ => bset![],
        }
    }

    fn exec(&self, regs: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        match self {
            CmpOp::GtA(sign_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_n2(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    match bool::from(sign_flag) {
                        true => val1.into_signed().cmp(&val2.into_signed()),
                        false => val1.cmp(&val2),
                    }
                }) == Some(Ordering::Greater);
            }
            CmpOp::GtF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_n2(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    if *eq_flag == FloatEqFlag::Rounding {
                        val1.rounding_cmp(&val2)
                    } else {
                        val1.cmp(&val2)
                    }
                }) == Some(Ordering::Greater);
            }
            CmpOp::GtR(reg, idx1, idx2) => {
                regs.st0 = regs.get_n2(reg, idx1, reg, idx2).map(|(val1, val2)| val1.cmp(&val2))
                    == Some(Ordering::Greater);
            }
            CmpOp::LtA(sign_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_n2(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    match bool::from(sign_flag) {
                        true => val1.into_signed().cmp(&val2.into_signed()),
                        false => val1.cmp(&val2),
                    }
                }) == Some(Ordering::Less);
            }
            CmpOp::LtF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs.get_n2(reg, idx1, reg, idx2).map(|(val1, val2)| {
                    if *eq_flag == FloatEqFlag::Rounding {
                        val1.rounding_cmp(&val2)
                    } else {
                        val1.cmp(&val2)
                    }
                }) == Some(Ordering::Less);
            }
            CmpOp::LtR(reg, idx1, idx2) => {
                regs.st0 = regs.get_n2(reg, idx1, reg, idx2).map(|(val1, val2)| val1.cmp(&val2))
                    == Some(Ordering::Less);
            }
            CmpOp::EqA(st, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_n2(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| val1 == val2)
                    .unwrap_or(*st == NoneEqFlag::Equal);
            }
            CmpOp::EqF(eq_flag, reg, idx1, idx2) => {
                regs.st0 = regs
                    .get_n2(reg, idx1, reg, idx2)
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
                    .get_n2(reg, idx1, reg, idx2)
                    .map(|(val1, val2)| val1 == val2)
                    .unwrap_or(*st == NoneEqFlag::Equal);
            }
            CmpOp::IfZA(reg, idx) => {
                regs.st0 = regs.get_n(reg, idx).map(Number::is_zero).unwrap_or(false)
            }
            CmpOp::IfZR(reg, idx) => {
                regs.st0 = regs.get_n(reg, idx).map(Number::is_zero).unwrap_or(false)
            }
            CmpOp::IfNA(reg, idx) => regs.st0 = regs.get_n(reg, idx).is_none(),
            CmpOp::IfNR(reg, idx) => regs.st0 = regs.get_n(reg, idx).is_none(),
            CmpOp::St(merge_flag, reg, idx) => {
                let st = Number::from(regs.st0 as u8);
                let res = match (*regs.get_n(reg, idx), merge_flag) {
                    (None, _) | (_, MergeFlag::Set) => st,
                    (Some(val), MergeFlag::Add) => {
                        val.int_add(st, IntFlags { signed: false, wrap: false }).unwrap_or(val)
                    }
                    (Some(val), MergeFlag::And) => val & st,
                    (Some(val), MergeFlag::Or) => val | st,
                };
                regs.set_n(reg, idx, Some(res));
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

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            ArithmeticOp::Neg(reg, idx) | ArithmeticOp::Abs(reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }
            ArithmeticOp::Stp(reg, idx, _) => {
                bset![Reg::A(*reg, *idx)]
            }
            ArithmeticOp::AddA(_, reg, src, srcdst)
            | ArithmeticOp::SubA(_, reg, src, srcdst)
            | ArithmeticOp::MulA(_, reg, src, srcdst)
            | ArithmeticOp::DivA(_, reg, src, srcdst) => {
                bset![Reg::A(*reg, *src), Reg::A(*reg, *srcdst)]
            }
            ArithmeticOp::AddF(_, reg, src, srcdst)
            | ArithmeticOp::SubF(_, reg, src, srcdst)
            | ArithmeticOp::MulF(_, reg, src, srcdst)
            | ArithmeticOp::DivF(_, reg, src, srcdst) => {
                bset![Reg::F(*reg, *src), Reg::F(*reg, *srcdst)]
            }
            ArithmeticOp::Rem(reg1, src, reg2, srcdst) => {
                bset![Reg::A(*reg1, *src), Reg::A(*reg2, *srcdst)]
            }
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            ArithmeticOp::Neg(reg, idx) | ArithmeticOp::Abs(reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }
            ArithmeticOp::Stp(reg, idx, _) => {
                bset![Reg::A(*reg, *idx)]
            }
            ArithmeticOp::AddA(_, reg, _src, srcdst)
            | ArithmeticOp::SubA(_, reg, _src, srcdst)
            | ArithmeticOp::MulA(_, reg, _src, srcdst)
            | ArithmeticOp::DivA(_, reg, _src, srcdst) => {
                bset![Reg::A(*reg, *srcdst)]
            }
            ArithmeticOp::AddF(_, reg, _src, srcdst)
            | ArithmeticOp::SubF(_, reg, _src, srcdst)
            | ArithmeticOp::MulF(_, reg, _src, srcdst)
            | ArithmeticOp::DivF(_, reg, _src, srcdst) => {
                bset![Reg::F(*reg, *srcdst)]
            }
            ArithmeticOp::Rem(_reg1, _src, reg2, srcdst) => {
                bset![Reg::A(*reg2, *srcdst)]
            }
        }
    }

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
                regs.set_n(reg, idx, regs.get_n(reg, idx).and_then(Number::abs))
            }
            ArithmeticOp::AddA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_add(val2, *flags));
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::AddF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_add(val2, *flags).into());
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::SubA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_sub(val2, *flags));
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::SubF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_sub(val2, *flags).into());
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::MulA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_mul(val2, *flags));
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::MulF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_mul(val2, *flags).into());
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::DivA(flags, reg, src, srcdst) => {
                let res = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.int_div(val2, *flags));
                regs.set_n(reg, srcdst, res)
            }
            ArithmeticOp::DivF(flags, reg, src, srcdst) => {
                let res: Option<Number> = regs
                    .get_n2(reg, src, reg, srcdst)
                    .and_then(|(val1, val2)| val1.float_div(val2, *flags).into());
                regs.set_n(reg, srcdst, res) && !res.map(Number::is_nan).unwrap_or(false)
            }
            ArithmeticOp::Rem(reg1, idx1, reg2, idx2) => {
                let res =
                    regs.get_n2(reg1, idx1, reg2, idx2).and_then(|(val1, val2)| val1.rem(val2));
                regs.set_n(reg2, idx2, res)
            }
            ArithmeticOp::Stp(reg, idx, step) => regs.set_n(
                reg,
                idx,
                regs.get_n(reg, idx).and_then(|val| {
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
                regs.set_n(reg, idx, regs.get_n(reg, idx).and_then(Number::neg))
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

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            BitwiseOp::And(reg, idx1, idx2, _idx3)
            | BitwiseOp::Or(reg, idx1, idx2, _idx3)
            | BitwiseOp::Xor(reg, idx1, idx2, _idx3) => {
                bset![Reg::new(*reg, *idx1), Reg::new(*reg, *idx2)]
            }
            BitwiseOp::Not(reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }

            BitwiseOp::Shl(a2, shift, reg, idx) => {
                bset![Reg::new(*a2, *shift), Reg::new(*reg, *idx)]
            }
            BitwiseOp::ShrA(_, a2, shift, reg, idx) => {
                bset![Reg::new(*a2, *shift), Reg::A(*reg, *idx)]
            }
            BitwiseOp::ShrR(a2, shift, reg, idx) => {
                bset![Reg::new(*a2, *shift), Reg::R(*reg, *idx)]
            }

            BitwiseOp::Scl(a2, shift, reg, idx) => {
                bset![Reg::new(*a2, *shift), Reg::new(*reg, *idx)]
            }
            BitwiseOp::Scr(a2, shift, reg, idx) => {
                bset![Reg::new(*a2, *shift), Reg::new(*reg, *idx)]
            }

            BitwiseOp::RevA(reg, idx) => {
                bset![Reg::A(*reg, *idx)]
            }
            BitwiseOp::RevR(reg, idx) => {
                bset![Reg::R(*reg, *idx)]
            }
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            BitwiseOp::And(reg, _idx1, _idx2, idx3)
            | BitwiseOp::Or(reg, _idx1, _idx2, idx3)
            | BitwiseOp::Xor(reg, _idx1, _idx2, idx3) => {
                bset![Reg::new(*reg, *idx3)]
            }
            BitwiseOp::Not(reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }

            BitwiseOp::Shl(_, _, reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }
            BitwiseOp::ShrA(_, _, _, reg, idx) => {
                bset![Reg::A(*reg, *idx)]
            }
            BitwiseOp::ShrR(_, _, reg, idx) => {
                bset![Reg::R(*reg, *idx)]
            }

            BitwiseOp::Scl(_, _, reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }
            BitwiseOp::Scr(_, _, reg, idx) => {
                bset![Reg::new(*reg, *idx)]
            }

            BitwiseOp::RevA(reg, idx) => {
                bset![Reg::A(*reg, *idx)]
            }
            BitwiseOp::RevR(reg, idx) => {
                bset![Reg::R(*reg, *idx)]
            }
        }
    }

    fn exec(&self, regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        fn shl(original: &[u8], shift: usize, n_bytes: usize) -> [u8; 1024] {
            let mut ret = [0u8; 1024];
            let word_shift = shift / 8;
            let bit_shift = shift % 8;
            for i in 0..n_bytes {
                // Shift
                if bit_shift < 8 && i + word_shift < n_bytes {
                    ret[i + word_shift] += original[i] << bit_shift;
                }
                // Carry
                if bit_shift > 0 && i + word_shift + 1 < n_bytes {
                    ret[i + word_shift + 1] += original[i] >> (8 - bit_shift);
                }
            }
            ret
        }
        fn shr(original: &[u8], shift: usize, n_bytes: usize) -> [u8; 1024] {
            let mut ret = [0u8; 1024];
            let word_shift = shift / 8;
            let bit_shift = shift % 8;
            for i in word_shift..n_bytes {
                // Shift
                ret[i - word_shift] += original[i] >> bit_shift;
                // Carry
                if bit_shift > 0 && i < n_bytes - 1 {
                    ret[i - word_shift] += original[i + 1] << (8 - bit_shift);
                }
            }
            ret
        }
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
                regs.set_n(reg, idx, !regs.get_n(reg, idx));
            }
            BitwiseOp::Shl(reg1, shift, reg2, srcdst) => match reg2 {
                RegAR::A(a) => {
                    let msb = regs.get_n(a, srcdst).unwrap_or_default()[a.bytes() - 1] & 0x80;
                    regs.st0 = msb == 0x80;
                    regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Shl::shl)
                }
                RegAR::R(r) => {
                    let shift = match reg1 {
                        RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as usize,
                        RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default() as usize,
                    };
                    if let Some(original) = regs.get_r_mut(*r, srcdst) {
                        let msb = original.last().copied().unwrap_or_default() & 0x80;
                        let n_bytes = reg2.bytes() as usize;
                        original.copy_from_slice(&shl(original, shift, n_bytes)[..n_bytes]);
                        regs.st0 = msb == 0x80;
                    }
                }
            },
            BitwiseOp::ShrA(flag, reg1, shift, reg2, srcdst) => {
                let res = regs.get_n2(reg1, shift, reg2, srcdst).map(|(shift, val)| {
                    let lsb = val[0] & 1;
                    regs.st0 = lsb == 1;
                    if *flag == SignFlag::Signed {
                        val.into_signed().shr(shift)
                    } else {
                        val.shr(shift)
                    }
                });
                regs.set_n(reg2, srcdst, res);
            }
            BitwiseOp::ShrR(reg1, shift, reg2, srcdst) => {
                let shift = match reg1 {
                    RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as usize,
                    RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default() as usize,
                };
                if let Some(original) = regs.get_r_mut(*reg2, srcdst) {
                    let lsb = original[0] & 1;
                    let n_bytes = reg2.bytes() as usize;
                    original.copy_from_slice(&shr(original, shift, n_bytes)[..n_bytes]);
                    regs.st0 = lsb == 1;
                }
            }
            BitwiseOp::Scl(reg1, shift, reg2, srcdst) => match reg2 {
                RegAR::A(_) => {
                    let msb = regs.get_n(reg2, srcdst).unwrap_or_default()[reg2.bytes() - 1] & 0x80;
                    regs.st0 = msb == 0x80;
                    regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Number::scl)
                }
                RegAR::R(r) => {
                    let shift = match reg1 {
                        RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as usize,
                        RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default() as usize,
                    };
                    let shift = shift % reg2.bits() as usize;
                    if let Some(original) = regs.get_r_mut(*r, srcdst) {
                        let msb = original.last().copied().unwrap_or_default() & 0x80;
                        let n_bytes = reg2.bytes() as usize;
                        let mut shl = shl(original, shift, n_bytes);
                        let shr = shr(original, reg2.bits() as usize - shift, n_bytes);
                        for i in 0..n_bytes {
                            shl[i] |= shr[i];
                        }
                        original.copy_from_slice(&shl[..n_bytes]);
                        regs.st0 = msb == 0x80;
                    }
                }
            },
            BitwiseOp::Scr(reg1, shift, reg2, srcdst) => match reg2 {
                RegAR::A(_) => {
                    let lsb = regs.get_n(reg2, srcdst).unwrap_or_default()[0] & 1;
                    regs.st0 = lsb == 1;
                    regs.op(reg2, srcdst, reg1, shift, reg2, srcdst, Number::scr)
                }
                RegAR::R(r) => {
                    let shift = match reg1 {
                        RegA2::A8 => regs.a8[shift.to_usize()].unwrap_or_default() as usize,
                        RegA2::A16 => regs.a16[shift.to_usize()].unwrap_or_default() as usize,
                    };
                    let shift = shift % reg2.bits() as usize;
                    if let Some(original) = regs.get_r_mut(*r, srcdst) {
                        let lsb = original[0] & 1;
                        let n_bytes = reg2.bytes() as usize;
                        let mut shr = shr(original, shift, n_bytes);
                        let shl = shl(original, reg2.bits() as usize - shift, n_bytes);
                        for i in 0..n_bytes {
                            shr[i] |= shl[i];
                        }
                        original.copy_from_slice(&shr[..n_bytes]);
                        regs.st0 = lsb == 1;
                    }
                }
            },
            BitwiseOp::RevA(reg, idx) => {
                regs.set_n(reg, idx, regs.get_n(reg, idx).map(Number::reverse_bits));
            }
            BitwiseOp::RevR(reg, idx) => {
                if let Some(original) = regs.get_r_mut(*reg, idx) {
                    original.reverse();
                    original.iter_mut().for_each(|byte| *byte = byte.reverse_bits());
                }
            }
        }
        ExecStep::Next
    }
}

impl InstructionSet for BytesOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            BytesOp::Put(_reg, _, _) => {
                bset![]
            }
            BytesOp::Swp(reg1, reg2) | BytesOp::Find(reg1, reg2) => {
                bset![Reg::S(*reg1), Reg::S(*reg2)]
            }
            BytesOp::Mov(reg1, _reg2) | BytesOp::Rev(reg1, _reg2) => {
                bset![Reg::S(*reg1)]
            }
            BytesOp::Fill(reg, offset1, offset2, value, _) => {
                bset![
                    Reg::S(*reg),
                    Reg::A(RegA::A16, *offset1),
                    Reg::A(RegA::A16, *offset2),
                    Reg::A(RegA::A8, *value)
                ]
            }
            BytesOp::Len(src, _reg, _dst) => {
                bset![Reg::S(*src)]
            }
            BytesOp::Cnt(src, byte, _cnt) => {
                bset![Reg::S(*src), Reg::new(RegA::A8, *byte)]
            }
            BytesOp::Eq(reg1, reg2) => {
                bset![Reg::S(*reg1), Reg::S(*reg2)]
            }
            BytesOp::Con(reg1, reg2, no, _offset, _len) => {
                bset![Reg::S(*reg1), Reg::S(*reg2), Reg::A(RegA::A16, *no),]
            }
            BytesOp::Extr(src, _dst, _index, offset) => {
                bset![Reg::S(*src), Reg::new(RegA::A16, *offset)]
            }
            BytesOp::Inj(src1, src2, index, offset) => {
                bset![Reg::S(*src1), Reg::new(*src2, *index), Reg::new(RegA::A16, *offset)]
            }
            BytesOp::Join(src1, src2, _dst) => {
                bset![Reg::S(*src1), Reg::S(*src2)]
            }
            BytesOp::Splt(_flag, offset, src, _dst1, _dst2) => {
                bset![Reg::A(RegA::A16, *offset), Reg::S(*src)]
            }
            BytesOp::Ins(_flag, offset, src, _dst) => {
                bset![Reg::A(RegA::A16, *offset), Reg::S(*src)]
            }
            BytesOp::Del(_flag, reg1, offset1, reg2, offset2, _flag1, _flag2, src, _dst) => {
                bset![Reg::new(*reg1, *offset1), Reg::new(*reg2, *offset2), Reg::S(*src)]
            }
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            BytesOp::Put(reg, _, _) => {
                bset![Reg::S(*reg)]
            }
            BytesOp::Swp(reg1, reg2) | BytesOp::Find(reg1, reg2) => {
                bset![Reg::S(*reg1), Reg::S(*reg2)]
            }
            BytesOp::Mov(_reg1, reg2) | BytesOp::Rev(_reg1, reg2) => {
                bset![Reg::S(*reg2)]
            }
            BytesOp::Fill(reg, _offset1, _offset2, _value, _) => {
                bset![Reg::S(*reg)]
            }
            BytesOp::Len(_src, reg, dst) => {
                bset![Reg::A(*reg, *dst)]
            }
            BytesOp::Cnt(_src, _byte, cnt) => {
                bset![Reg::new(RegA::A16, *cnt)]
            }
            BytesOp::Eq(_reg1, _reg2) => {
                bset![]
            }
            BytesOp::Con(_reg1, _reg2, _no, offset, len) => {
                bset![Reg::A(RegA::A16, *offset), Reg::A(RegA::A16, *len)]
            }
            BytesOp::Extr(_src, dst, index, _offset) => {
                bset![Reg::new(*dst, *index)]
            }
            BytesOp::Inj(src1, _src2, _index, _offset) => {
                bset![Reg::S(*src1)]
            }
            BytesOp::Join(_src1, _src2, dst) => {
                bset![Reg::S(*dst)]
            }
            BytesOp::Splt(_flag, _offset, _src, dst1, dst2) => {
                bset![Reg::S(*dst1), Reg::S(*dst2)]
            }
            BytesOp::Ins(_flag, _offset, _src, dst) => {
                bset![Reg::S(*dst)]
            }
            BytesOp::Del(_flag, _reg1, _offset1, _reg2, _offset2, _flag1, _flag2, _src, dst) => {
                bset![Reg::S(*dst)]
            }
        }
    }

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
                    regs.set_n(reg, dst, len as u32);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set_n(reg, dst, MaybeNumber::none());
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
                    regs.set_n(RegA::A16, dst, count as u32);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set_n(RegA::A16, dst, MaybeNumber::none());
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
                    let (s1, s2) = regs.get_s2(*reg1, *reg2)?;
                    let r1 = s1.as_ref();
                    let r2 = s2.as_ref();
                    let count = r1.windows(r2.len()).filter(|r1| *r1 == r2).count();
                    assert!(count <= u16::MAX as usize);
                    regs.set_n(RegA::A16, Reg32::Reg0, count as u16);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set_n(RegA::A16, Reg32::Reg0, MaybeNumber::none());
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
                    regs.set_n(RegA::A16, offset_dst, offset);
                    regs.set_n(RegA::A16, len_dst, len);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set_n(RegA::A16, offset_dst, MaybeNumber::none());
                    regs.set_n(RegA::A16, len_dst, MaybeNumber::none());
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
                    regs.set_n(dst, index, num);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set_n(dst, index, MaybeNumber::none());
                })
            }
            BytesOp::Inj(src, dst, index, offset) => {
                let mut f = || -> Option<()> {
                    let mut s = regs.get_s(*src)?.clone();
                    let val = regs.get_n(dst, index).map(|v| v)?;
                    let offset = regs.a16[*offset as u8 as usize]?;
                    let end = offset.saturating_add(dst.layout().bytes() - 1);
                    s.adjust_len(end);
                    s.as_mut()[offset as usize..=end as usize].copy_from_slice(val.as_ref());
                    regs.s16[src.as_usize()] = Some(s);
                    Some(())
                };
                f().unwrap_or_else(|| {
                    regs.st0 = false;
                    regs.set_n(dst, index, MaybeNumber::none());
                })
            }
            BytesOp::Join(src1, src2, dst) => {
                let mut f = || -> Option<()> {
                    let (s1, s2) = regs.get_s2(*src1, *src2)?;
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

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            DigestOp::Ripemd(src, _dst)
            | DigestOp::Sha256(src, _dst)
            | DigestOp::Sha512(src, _dst) => bset![Reg::S(*src)],
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            DigestOp::Ripemd(_src, dst) => bset![Reg::new(RegR::R160, *dst)],
            DigestOp::Sha256(_src, dst) => bset![Reg::new(RegR::R256, *dst)],
            DigestOp::Sha512(_src, dst) => bset![Reg::new(RegR::R512, *dst)],
        }
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
                regs.set_n(RegR::R160, dst, hash);
            }
            DigestOp::Sha256(src, dst) => {
                let s = regs.get_s(*src);
                none = s.is_none();
                let hash: Option<[u8; 32]> = s.map(|s| sha2::Sha256::digest(s.as_ref()).into());
                regs.set_n(RegR::R256, dst, hash);
            }
            DigestOp::Sha512(src, dst) => {
                let s = regs.get_s(*src);
                none = s.is_none();
                let hash: Option<[u8; 64]> = s.map(|s| sha2::Sha512::digest(s.as_ref()).into());
                regs.set_n(RegR::R512, dst, hash);
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

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            Secp256k1Op::Gen(src, _dst) => {
                bset![Reg::R(RegR::R256, *src)]
            }
            Secp256k1Op::Mul(RegBlockAR::A, scal, src, _dst) => {
                bset![Reg::A(RegA::A256, *scal), Reg::R(RegR::R512, *src)]
            }
            Secp256k1Op::Mul(RegBlockAR::R, scal, src, _dst) => {
                bset![Reg::R(RegR::R256, *scal), Reg::R(RegR::R512, *src)]
            }
            Secp256k1Op::Add(src, srcdst) => {
                bset![Reg::R(RegR::R512, *src), Reg::new(RegR::R512, *srcdst)]
            }
            Secp256k1Op::Neg(src, _dst) => {
                bset![Reg::R(RegR::R512, *src)]
            }
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            Secp256k1Op::Gen(_src, dst) => {
                bset![Reg::new(RegR::R512, *dst)]
            }
            Secp256k1Op::Mul(_, _, _src, dst) => {
                bset![Reg::R(RegR::R512, *dst)]
            }
            Secp256k1Op::Add(_src, srcdst) => {
                bset![Reg::new(RegR::R512, *srcdst)]
            }
            Secp256k1Op::Neg(_src, dst) => {
                bset![Reg::new(RegR::R512, *dst)]
            }
        }
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
                    .get_n(RegR::R256, src)
                    .and_then(|mut src| {
                        let src = src.as_mut();
                        // little endian to big endian
                        src.reverse();
                        SecretKey::from_slice(src).ok()
                    })
                    .map(|sk| PublicKey::from_secret_key(SECP256K1, &sk))
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set_n(RegR::R512, dst, res);
            }

            Secp256k1Op::Mul(block, scal, src, dst) => {
                let reg = block.into_reg(256).expect("register set does not match standard");
                let res = regs
                    .get_n(reg, scal)
                    .and_then(|scal| {
                        regs.get_n(RegR::R512, src)
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
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set_n(RegR::R512, dst, res);
            }

            Secp256k1Op::Add(src, srcdst) => {
                let res = regs
                    .get_n(RegR::R512, src)
                    .and_then(|val| {
                        let mut pk1 = [4u8; 65];
                        pk1[1..].copy_from_slice(val.as_ref());
                        PublicKey::from_slice(&pk1).ok()
                    })
                    .and_then(|pk1| {
                        regs.get_n(RegR::R512, srcdst).and_then(|val| {
                            let mut pk2 = [4u8; 65];
                            pk2[1..].copy_from_slice(val.as_ref());
                            PublicKey::from_slice(&pk2).ok().map(|pk2| (pk1, pk2))
                        })
                    })
                    .and_then(|(pk1, pk2)| pk1.combine(&pk2).ok())
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set_n(RegR::R512, srcdst, res);
            }

            Secp256k1Op::Neg(src, dst) => {
                let res = regs
                    .get_n(RegR::R512, src)
                    .and_then(|val| {
                        let mut pk = [4u8; 65];
                        pk[1..].copy_from_slice(&val[..]);
                        PublicKey::from_slice(&pk).ok()
                    })
                    .map(|pk| pk.negate(SECP256K1))
                    .as_ref()
                    .map(PublicKey::serialize_uncompressed)
                    .map(|pk| Number::from_slice(&pk[1..]));
                regs.set_n(RegR::R512, dst, res);
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

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            Curve25519Op::Gen(src, _dst) => {
                bset![Reg::R(RegR::R256, *src)]
            }
            Curve25519Op::Mul(RegBlockAR::A, scal, src, _dst) => {
                bset![Reg::A(RegA::A256, *scal), Reg::R(RegR::R512, *src)]
            }
            Curve25519Op::Mul(RegBlockAR::R, scal, src, _dst) => {
                bset![Reg::R(RegR::R256, *scal), Reg::R(RegR::R512, *src)]
            }
            Curve25519Op::Add(src1, src2, _dst, _) => {
                bset![Reg::R(RegR::R512, *src1), Reg::new(RegR::R512, *src2)]
            }
            Curve25519Op::Neg(src, _dst) => {
                bset![Reg::R(RegR::R512, *src)]
            }
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            Curve25519Op::Gen(_src, dst) => {
                bset![Reg::new(RegR::R512, *dst)]
            }
            Curve25519Op::Mul(_, _, _src, dst) => {
                bset![Reg::R(RegR::R512, *dst)]
            }
            Curve25519Op::Add(_src1, _src2, dst, _) => {
                bset![Reg::new(RegR::R512, *dst)]
            }
            Curve25519Op::Neg(_src, dst) => {
                bset![Reg::new(RegR::R512, *dst)]
            }
        }
    }

    #[inline]
    fn complexity(&self) -> u64 { 1000 }

    #[cfg(not(feature = "curve25519"))]
    fn exec(&self, _: &mut CoreRegs, _: LibSite, _: &()) -> ExecStep {
        unimplemented!("AluVM runtime compiled without support for Curve25519 instructions")
    }

    #[cfg(feature = "curve25519")]
    fn exec(&self, _regs: &mut CoreRegs, _site: LibSite, _: &()) -> ExecStep {
        todo!("implement Curve256 operations")
    }
}

impl InstructionSet for ReservedOp {
    type Context<'ctx> = ();

    #[inline]
    fn isa_ids() -> BTreeSet<&'static str> { BTreeSet::default() }

    fn src_regs(&self) -> BTreeSet<Reg> { bset![] }

    fn dst_regs(&self) -> BTreeSet<Reg> { bset![] }

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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1).unwrap(), Number::from(0u16));
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2).unwrap(), Number::from(5u16));
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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1).unwrap(), Number::from(6u16));
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2).unwrap(), Number::from(6u16));
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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1).unwrap(), Number::from(13u16));
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2).unwrap(), Number::from(4u16));
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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1), MaybeNumber::none());
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2), MaybeNumber::none());
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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1), MaybeNumber::none());
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2), MaybeNumber::none());
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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1).unwrap(), Number::from(0u16));
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2).unwrap(), Number::from(u16::MAX));
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
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg1), MaybeNumber::none());
        assert_eq!(register.get_n(RegA::A16, Reg32::Reg2), MaybeNumber::none());
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
        assert!(register.st0);
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
        assert!(register.st0);
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
        assert!(!register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());
        assert!(register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R512, Reg32::Reg0, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
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
        assert!(register.st0);
    }

    /* TODO: Enable after curve25519 re-implementation
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
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg1, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg0, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(!register.st0);
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
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg2, Reg32::Reg3).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
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
        Curve25519Op::Gen(Reg32::Reg0, Reg8::Reg7).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg1, Reg8::Reg1).exec(&mut register, lib_site, &());
        Curve25519Op::Gen(Reg32::Reg2, Reg8::Reg2).exec(&mut register, lib_site, &());
        Curve25519Op::Add(Reg32::Reg7, Reg32::Reg1, Reg32::Reg3, false).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(!register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());
        Curve25519Op::Add(Reg32::Reg0, Reg32::Reg1, Reg32::Reg3, true).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg2, Reg32::Reg3).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
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
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg0, Reg32::Reg1).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(!register.st0);
        ControlFlowOp::Succ.exec(&mut register, lib_site, &());
        assert!(register.st0);
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg0, Reg32::Reg2).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
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
        CmpOp::EqR(NoneEqFlag::NonEqual, RegR::R256, Reg32::Reg4, Reg32::Reg6).exec(
            &mut register,
            lib_site,
            &(),
        );
        assert!(register.st0);
    }
     */
}
