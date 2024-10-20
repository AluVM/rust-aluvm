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

use crate::core::{AluCore, Reg, Site};

/// Turing machine movement after instruction execution
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ExecStep<Site> {
    /// Stop program execution
    Stop,

    /// Stop and fail program execution
    Fail,

    /// Move to the next instruction
    Next,

    /// Jump to the offset from the origin
    Jump(u16),

    /// Jump to another code fragment
    Call(Site),
}

/// Trait for instructions
pub trait Instruction: core::fmt::Display + core::fmt::Debug {
    /// Context: external data which are accessible to the ISA.
    type Context<'ctx>;

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

    /// The size of the data coming as an instruction operands (i.e. except data coming from
    /// registers or read from outside the instruction operands).
    fn op_data_size(&self) -> u16;

    /// The size of the data read by the instruction from outside the registers (except data coming
    /// as a parameter).
    fn ext_data_size(&self) -> u16;

    /// Returns computational complexity of the instruction.
    ///
    /// Computational complexity is the number of "CPU ticks" required to process the instruction.
    fn complexity(&self) -> u64 {
        // By default, give the upper estimate
        self.op_data_size() as u64 * 8_000 // 1k of complexity units per input bit
        + self.src_regs()
            .iter()
            .chain(&self.dst_regs())
            .map(|reg| reg.bytes() as u64)
            .sum::<u64>()
            * 80_000 // 10k of complexity units per input and output bit
        + self.ext_data_size() as u64 * 800_000 // x10 complexity units per byte of external memory
    }

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
    fn exec<Id>(&self, regs: &mut AluCore<Id>, site: Site<Id>, context: &Self::Context<'_>) -> ExecStep<Site<Id>>;
}
