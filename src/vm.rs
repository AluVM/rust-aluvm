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

//! Alu virtual machine

use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::isa::{Instr, InstructionSet, ReservedOp};
use crate::library::LibSite;
use crate::reg::CoreRegs;
use crate::Program;

/// Alu virtual machine providing single-core execution environment
#[derive(Getters, Debug, Default)]
pub struct Vm<Isa = Instr<ReservedOp>>
where
    Isa: InstructionSet,
{
    /// A set of registers
    registers: Box<CoreRegs>,

    phantom: PhantomData<Isa>,
}

/// Runtime for program execution.
impl<Isa> Vm<Isa>
where
    Isa: InstructionSet,
{
    /// Constructs new virtual machine instance.
    pub fn new() -> Self { Self { registers: Box::default(), phantom: Default::default() } }

    /// Executes the program starting from the provided entry point (set with
    /// [`Program::set_entrypoint`] and [`Program::with`], or initialized to 0 offset of the
    /// first used library if [`Program::new`] was used).
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    pub fn run(&mut self, program: &impl Program<Isa = Isa>) -> bool {
        self.call(program, program.entrypoint())
    }

    /// Executes the program starting from the provided entry point.
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    pub fn call(&mut self, program: &impl Program<Isa = Isa>, method: LibSite) -> bool {
        let mut call = Some(method);
        while let Some(ref mut site) = call {
            if let Some(lib) = program.lib(site.lib) {
                call = lib.exec::<Isa>(site.pos, &mut self.registers);
            } else if let Some(pos) = site.pos.checked_add(1) {
                site.pos = pos;
            } else {
                call = None;
            };
        }
        self.registers.st0
    }
}
