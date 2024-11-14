// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 LNP/BP Standards Association, Switzerland.
// Copyright (C) 2024-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2021-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use alloc::collections::BTreeSet;

use super::CtrlInstr;
use crate::core::{Core, NoExt, NoRegs, Site, SiteId, Status};
use crate::isa::{ExecStep, Instr, Instruction, ReservedInstr};

impl<Id: SiteId> Instruction<Id> for Instr<Id> {
    const ISA_EXT: &'static [&'static str] = &[];

    type Core = NoExt;
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<NoRegs> {
        match self {
            Instr::Ctrl(instr) => instr.src_regs(),
            Instr::Reserved(instr) => Instruction::<Id>::src_regs(instr),
        }
    }

    fn dst_regs(&self) -> BTreeSet<NoRegs> {
        match self {
            Instr::Ctrl(instr) => instr.dst_regs(),
            Instr::Reserved(instr) => Instruction::<Id>::dst_regs(instr),
        }
    }

    fn op_data_bytes(&self) -> u16 {
        match self {
            Instr::Ctrl(instr) => instr.op_data_bytes(),
            Instr::Reserved(instr) => Instruction::<Id>::op_data_bytes(instr),
        }
    }

    fn ext_data_bytes(&self) -> u16 {
        match self {
            Instr::Ctrl(instr) => instr.ext_data_bytes(),
            Instr::Reserved(instr) => Instruction::<Id>::ext_data_bytes(instr),
        }
    }

    fn exec(
        &self,
        core: &mut Core<Id, Self::Core>,
        site: Site<Id>,
        _: &Self::Context<'_>,
    ) -> ExecStep<Site<Id>> {
        match self {
            Instr::Ctrl(instr) => instr.exec(core, site, &()),
            Instr::Reserved(instr) => instr.exec(core, site, &()),
        }
    }
}

impl<Id: SiteId> Instruction<Id> for ReservedInstr {
    const ISA_EXT: &'static [&'static str] = &[];

    type Core = NoExt;
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<NoRegs> { none!() }

    fn dst_regs(&self) -> BTreeSet<NoRegs> { none!() }

    fn op_data_bytes(&self) -> u16 { none!() }

    fn ext_data_bytes(&self) -> u16 { none!() }

    fn complexity(&self) -> u64 { u64::MAX }

    fn exec(
        &self,
        _: &mut Core<Id, Self::Core>,
        _: Site<Id>,
        _: &Self::Context<'_>,
    ) -> ExecStep<Site<Id>> {
        ExecStep::FailHalt
    }
}

impl<Id: SiteId> Instruction<Id> for CtrlInstr<Id> {
    const ISA_EXT: &'static [&'static str] = &[];

    type Core = NoExt;
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<NoRegs> { none!() }

    fn dst_regs(&self) -> BTreeSet<NoRegs> { none!() }

    fn op_data_bytes(&self) -> u16 {
        match self {
            CtrlInstr::Nop
            | CtrlInstr::Chk
            | CtrlInstr::NotCo
            | CtrlInstr::FailCk
            | CtrlInstr::RsetCk => 0,
            CtrlInstr::Jmp { .. } | CtrlInstr::JiNe { .. } | CtrlInstr::JiFail { .. } => 2,
            CtrlInstr::Sh { .. } | CtrlInstr::ShNe { .. } | CtrlInstr::ShFail { .. } => 1,
            CtrlInstr::Exec { .. } => 2,
            CtrlInstr::Fn { .. } => 2,
            CtrlInstr::Call { .. } => 2,
            CtrlInstr::Ret | CtrlInstr::Stop => 0,
        }
    }

    fn ext_data_bytes(&self) -> u16 {
        match self {
            CtrlInstr::Nop
            | CtrlInstr::Chk
            | CtrlInstr::NotCo
            | CtrlInstr::FailCk
            | CtrlInstr::RsetCk => 0,
            CtrlInstr::Jmp { .. } | CtrlInstr::JiNe { .. } | CtrlInstr::JiFail { .. } => 0,
            CtrlInstr::Sh { .. } | CtrlInstr::ShNe { .. } | CtrlInstr::ShFail { .. } => 0,
            CtrlInstr::Exec { .. } => 32,
            CtrlInstr::Fn { .. } => 0,
            CtrlInstr::Call { .. } => 32,
            CtrlInstr::Ret | CtrlInstr::Stop => 0,
        }
    }

    fn exec(
        &self,
        core: &mut Core<Id, Self::Core>,
        current: Site<Id>,
        _: &Self::Context<'_>,
    ) -> ExecStep<Site<Id>> {
        let shift_jump = |shift: i8| {
            let Some(pos) = current.offset.checked_add_signed(shift as i16) else {
                return ExecStep::FailHalt;
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
            CtrlInstr::JiNe { pos } => {
                if core.co() {
                    return ExecStep::Jump(pos);
                }
            }
            CtrlInstr::JiFail { pos } => {
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
                    None => ExecStep::FailHalt,
                }
            }
            CtrlInstr::Call { site } => {
                return match core.push_cs(current) {
                    Some(_) => ExecStep::Call(site),
                    None => ExecStep::FailHalt,
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
