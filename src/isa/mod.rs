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

//! AluVM instruction set architecture.

#[macro_use]
mod macros;
mod instr;
mod bytecode;
mod arch;

mod alu;
#[cfg(feature = "GFA")]
mod gfa;

pub use alu::{CtrlInstr, RegInstr};
pub use arch::{Instr, InstructionSet, IsaId, ReservedInstr, ISA_ALU128, ISA_ALU64, ISA_AN, ISA_ID_MAX_LEN};
pub use bytecode::{Bytecode, BytecodeRead, BytecodeWrite, CodeEofError};
#[cfg(feature = "GFA")]
pub use gfa::{FieldInstr, ISA_GFA128};
pub use instr::{ExecStep, Instruction};
