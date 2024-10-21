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

//! Alu virtual machine

use core::marker::PhantomData;

use crate::core::{AluCore, CoreConfig, Status};
use crate::isa::{Bytecode, Instr, Instruction, InstructionSet, ReservedInstr};
use crate::library::{Lib, LibId, LibSite};

/// Alu virtual machine providing single-core execution environment
#[derive(Clone, Debug)]
pub struct Vm<Isa = Instr<LibId, ReservedInstr>>
where Isa: InstructionSet<LibId>
{
    /// A set of registers
    pub registers: AluCore<LibId>,

    phantom: PhantomData<Isa>,
}

/// Runtime for program execution.
impl<Isa> Vm<Isa>
where Isa: InstructionSet<LibId>
{
    /// Constructs new virtual machine instance with default core configuration.
    pub fn new() -> Self {
        Self {
            registers: AluCore::new(),
            phantom: Default::default(),
        }
    }

    /// Constructs new virtual machine instance with default core configuration.
    pub fn with(config: CoreConfig) -> Self {
        Self {
            registers: AluCore::with(config),
            phantom: Default::default(),
        }
    }

    /// Executes the program starting from the provided entry point.
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    pub fn exec<'prog>(
        &mut self,
        entry_point: LibSite,
        lib_resolver: impl Fn(LibId) -> Option<&'prog Lib>,
        context: &<Isa::Instr as Instruction<LibId>>::Context<'_>,
    ) -> Status
    where
        Isa::Instr: Bytecode<LibId>,
    {
        let mut call = Some(entry_point);
        while let Some(ref mut site) = call {
            if let Some(lib) = lib_resolver(site.lib_id) {
                call = lib.exec::<Isa::Instr>(site.offset, &mut self.registers, context);
            } else if let Some(pos) = site.offset.checked_add(1) {
                site.offset = pos;
            } else {
                call = None;
            };
        }
        self.registers.ck()
    }
}
