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

use std::collections::BTreeSet;

use super::FieldInstr;
use crate::core::{Reg, RegA, SiteId};
use crate::isa::{ExecStep, Instruction};
use crate::{AluCore, Site};

macro_rules! A {
    [$reg:ident @ $core:ident] => {{
        let Some(val) = $core.a($reg) else {
            return ExecStep::NextFail;
        };
        val
    }};
    [$a:ident : $idx:ident @ $core:ident] => {{
        let Some(val) = $core.a(RegA::with($a.a(), $idx)) else {
            return ExecStep::NextFail;
        };
        val
    }};
}

macro_rules! check {
    ($condition:expr) => {{
        if !($condition) {
            return ExecStep::NextFail;
        }
    }};
}

impl<Id: SiteId> Instruction<Id> for FieldInstr {
    type Context<'ctx> = ();

    fn src_regs(&self) -> BTreeSet<Reg> { todo!() }

    fn dst_regs(&self) -> BTreeSet<Reg> { todo!() }

    fn op_data_size(&self) -> u16 { todo!() }

    fn ext_data_size(&self) -> u16 { todo!() }

    fn exec(&self, core: &mut AluCore<Id>, site: Site<Id>, context: &Self::Context<'_>) -> ExecStep<Site<Id>> {
        #[inline]
        fn add_mod(a: u128, b: u128, order: impl Into<u128>) -> Option<(u128, bool)> {
            let order = order.into();
            if a >= order || b >= order {
                return None;
            }
            let (mut res, overflow) = a.overflowing_add(b);
            if overflow {
                res = res + u128::MAX - order;
            }
            Some((res, overflow))
        }

        match *self {
            FieldInstr::IncMod { src_dst, val, order } => {
                let src = A![src_dst @ core];
                let res = add_mod(src, val as u128, order);
                let Some((res, overflow)) = res else {
                    return ExecStep::NextFail;
                };
                core.set_co(overflow);
                core.set_a(src_dst, res);
            }
            FieldInstr::DecMod { src_dst, val, order } => {
                let src = A![src_dst @ core];
                // negate val
                let val = order.to_u128() - val as u128;
                let res = add_mod(src, val, order);
                let Some((res, overflow)) = res else {
                    return ExecStep::NextFail;
                };
                core.set_co(overflow);
                core.set_a(src_dst, res);
            }
            FieldInstr::AddMod { src_dst, src, order } => {
                let src1 = A![src_dst @ core];
                let src2 = A![src_dst : src @ core];
                let res = add_mod(src1, src2, order);
                let Some((res, overflow)) = res else {
                    return ExecStep::NextFail;
                };
                core.set_co(overflow);
                core.set_a(src_dst, res);
            }
            FieldInstr::NegMod { dst, src, order } => {
                let src = A![dst : src @ core];
                let order = order.to_u128();
                check!(src < order);
                core.set_a(dst, order - src);
            }
            FieldInstr::MulMod { src_dst, src, order } => {
                let src1 = A![src_dst @ core];
                let src2 = A![src_dst : src @ core];

                // negate src2
                let order = order.to_u128();
                check!(src2 < order);
                let src2 = order - src2;

                let res = add_mod(src1, src2, order);
                let Some((res, overflow)) = res else {
                    return ExecStep::NextFail;
                };
                core.set_co(overflow);
                core.set_a(src_dst, res);
            }
        }
        ExecStep::Next
    }
}
