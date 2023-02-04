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

//! AluVM registers system

mod core_regs;
mod families;
mod indexes;

pub use core_regs::{CoreRegs, CALL_STACK_SIZE};
pub use families::{
    NumericRegister, RegA, RegA2, RegAF, RegAFR, RegAR, RegAll, RegBlock, RegBlockAFR, RegBlockAR,
    RegF, RegR,
};
pub use indexes::{Reg16, Reg32, Reg8, RegS};

/// Trait marking all types representing register family, specific register or register index
pub trait Register: Default {
    /// Text description of the register family
    fn description() -> &'static str;
}
