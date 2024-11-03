// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 UBIDECO Labs,
//     Laboratories for Distributed and Cognitive Computing, Switzerland.
//     All rights reserved.
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

use alloc::collections::BTreeSet;

use super::{CtrlInstr, MaybeU128, RegInstr};
use crate::core::{Core, Reg, Site, SiteId, Status};
use crate::isa::{ExecStep, Instr, Instruction, InstructionSet, ReservedInstr};

impl<Id: SiteId, Ext: InstructionSet<Id> + Instruction<Id>> Instruction<Id> for Instr<Id, Ext> {
    type Context<'ctx> = Ext::Context<'ctx>;

    fn src_regs(&self) -> BTreeSet<Reg> {
        match self {
            Instr::Ctrl(instr) => instr.src_regs(),
            Instr::Reg(instr) => Instruction::<Id>::src_regs(instr),
            #[cfg(feature = "GFA")]
            Instr::GFqA(instr) => Instruction::<Id>::src_regs(instr),
            Instr::Reserved(instr) => Instruction::<Id>::src_regs(instr),
            Instr::Ext(instr) => instr.src_regs(),
        }
    }

    fn dst_regs(&self) -> BTreeSet<Reg> {
        match self {
            Instr::Ctrl(instr) => instr.dst_regs(),
            Instr::Reg(instr) => Instruction::<Id>::dst_regs(instr),
            #[cfg(feature = "GFA")]
            Instr::GFqA(instr) => Instruction::<Id>::dst_regs(instr),
            Instr::Reserved(instr) => Instruction::<Id>::dst_regs(instr),
            Instr::Ext(instr) => instr.dst_regs(),
        }
    }

    fn op_data_bytes(&self) -> u16 {
        match self {
            Instr::Ctrl(instr) => instr.op_data_bytes(),
            Instr::Reg(instr) => Instruction::<Id>::op_data_bytes(instr),
            #[cfg(feature = "GFA")]
            Instr::GFqA(instr) => Instruction::<Id>::op_data_bytes(instr),
            Instr::Reserved(instr) => Instruction::<Id>::op_data_bytes(instr),
            Instr::Ext(instr) => instr.op_data_bytes(),
        }
    }

    fn ext_data_bytes(&self) -> u16 {
        match self {
            Instr::Ctrl(instr) => instr.ext_data_bytes(),
            Instr::Reg(instr) => Instruction::<Id>::ext_data_bytes(instr),
            #[cfg(feature = "GFA")]
            Instr::GFqA(instr) => Instruction::<Id>::ext_data_bytes(instr),
            Instr::Reserved(instr) => Instruction::<Id>::ext_data_bytes(instr),
            Instr::Ext(instr) => instr.ext_data_bytes(),
        }
    }

    fn exec(&self, core: &mut Core<Id>, site: Site<Id>, context: &Self::Context<'_>) -> ExecStep<Site<Id>> {
        match self {
            Instr::Ctrl(instr) => instr.exec(core, site, &()),
            Instr::Reg(instr) => instr.exec(core, site, &()),
            #[cfg(feature = "GFA")]
            Instr::GFqA(instr) => instr.exec(core, site, &()),
            Instr::Reserved(instr) => instr.exec(core, site, &()),
            Instr::Ext(instr) => instr.exec(core, site, context),
        }
    }
}

impl<Id: SiteId> Instruction<Id> for ReservedInstr {
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<Reg> { none!() }

    fn dst_regs(&self) -> BTreeSet<Reg> { none!() }

    fn op_data_bytes(&self) -> u16 { none!() }

    fn ext_data_bytes(&self) -> u16 { none!() }

    fn complexity(&self) -> u64 { u64::MAX }

    fn exec(&self, _: &mut Core<Id>, _: Site<Id>, _: &Self::Context<'_>) -> ExecStep<Site<Id>> { ExecStep::StopFail }
}

impl<Id: SiteId> Instruction<Id> for CtrlInstr<Id> {
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<Reg> { none!() }

    fn dst_regs(&self) -> BTreeSet<Reg> { none!() }

    fn op_data_bytes(&self) -> u16 { todo!() }

    fn ext_data_bytes(&self) -> u16 { todo!() }

    fn exec(&self, core: &mut Core<Id>, current: Site<Id>, _: &Self::Context<'_>) -> ExecStep<Site<Id>> {
        let shift_jump = |shift: i8| {
            let Some(pos) = current.offset.checked_add_signed(shift as i16) else {
                return ExecStep::StopFail;
            };
            return ExecStep::Jump(pos);
        };

        match *self {
            CtrlInstr::Nop => {}
            CtrlInstr::Chk => {
                if core.ck() == Status::Fail {
                    return ExecStep::Stop;
                }
            }
            CtrlInstr::FailCk => {
                if core.fail_ck() {
                    return ExecStep::Stop;
                }
            }
            CtrlInstr::RsetCk => {
                core.set_co(core.ck() == Status::Fail);
                core.reset_ck()
            }
            CtrlInstr::NotCo => core.set_co(!core.co()),
            CtrlInstr::Jmp { pos } => return ExecStep::Jump(pos),
            CtrlInstr::JifCo { pos } => {
                if core.co() {
                    return ExecStep::Jump(pos);
                }
            }
            CtrlInstr::JifCk { pos } => {
                if core.ck() == Status::Fail {
                    return ExecStep::Jump(pos);
                }
            }
            CtrlInstr::Sh { shift } => {
                return shift_jump(shift);
            }
            CtrlInstr::ShNe { shift } => {
                if core.co() {
                    return shift_jump(shift);
                }
            }
            CtrlInstr::ShFail { shift } => {
                if core.ck() == Status::Fail {
                    return shift_jump(shift);
                }
            }
            CtrlInstr::Exec { site } => return ExecStep::Call(site),
            CtrlInstr::Fn { pos } => {
                return match core.push_cs(current) {
                    Some(_) => ExecStep::Jump(pos),
                    None => ExecStep::StopFail,
                }
            }
            CtrlInstr::Call { site } => {
                return match core.push_cs(current) {
                    Some(_) => ExecStep::Call(site),
                    None => ExecStep::StopFail,
                }
            }
            CtrlInstr::Ret => {
                return match core.pop_cs() {
                    Some(site) => ExecStep::Call(site),
                    None => ExecStep::Stop,
                }
            }
            CtrlInstr::Stop => return ExecStep::Stop,
        }
        ExecStep::Next
    }
}

impl<Id: SiteId> Instruction<Id> for RegInstr {
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<Reg> { todo!() }

    fn dst_regs(&self) -> BTreeSet<Reg> { todo!() }

    fn op_data_bytes(&self) -> u16 { todo!() }

    fn ext_data_bytes(&self) -> u16 { todo!() }

    fn exec(&self, core: &mut Core<Id>, _: Site<Id>, _: &Self::Context<'_>) -> ExecStep<Site<Id>> {
        match *self {
            RegInstr::Clr { dst } => {
                let was_set = core.clr_a(dst);
                core.set_co(was_set);
            }
            RegInstr::Put {
                dst: _,
                val: MaybeU128::NoData,
            }
            | RegInstr::Pif {
                dst: _,
                val: MaybeU128::NoData,
            } => {
                if core.fail_ck() {
                    return ExecStep::Stop;
                }
            }
            RegInstr::Put {
                dst,
                val: MaybeU128::U128(val),
            } => {
                let was_set = core.set_a(dst, val);
                core.set_co(was_set);
            }
            RegInstr::Pif {
                dst,
                val: MaybeU128::U128(val),
            } => {
                if core.a(dst).is_none() {
                    let was_set = core.set_a(dst, val);
                    core.set_co(was_set);
                }
            }
            RegInstr::Test { src } => {
                let was_set = core.a(src).is_some();
                core.set_co(was_set);
            }
            RegInstr::Cpy { dst, src } => {
                let was_set = match core.a(src) {
                    None => core.clr_a(dst),
                    Some(val) => core.set_a(dst, val),
                };
                core.set_co(was_set);
            }
            RegInstr::Swp { src_dst1, src_dst2 } => match core.take_a(src_dst1) {
                Some(a) => {
                    core.swp_a(src_dst2, a).map(|b| core.set_a(src_dst1, b));
                }
                None => {
                    core.clr_a(src_dst1);
                }
            },
            RegInstr::Eq { src1, src2 } => {
                let a = core.a(src1);
                let b = core.a(src2);
                core.set_co(a == b);
            }
        }
        ExecStep::Next
    }
}
