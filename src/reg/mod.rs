// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Labs,
//     Institute for Distributed and Cognitive Computing, Switzerland.
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

/// Superset of all registers accessible via instructions. The superset includes `A`, `F`, `R` and
/// `S` families of registers.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From)]
pub enum Reg {
    /// Arithmetic integer registers (`A` registers)
    #[display("{0}{1}")]
    A(RegA, Reg32),

    /// Arithmetic float registers (`F` registers)
    #[display("{0}{1}")]
    F(RegF, Reg32),

    /// Non-arithmetic (general) registers (`R` registers)
    #[display("{0}{1}")]
    R(RegR, Reg32),

    /// String registers (`S` registers)
    #[display("{0}")]
    #[from]
    S(RegS),
}

impl Reg {
    /// Construct register information
    pub fn new(reg: impl Into<RegAFR>, index: impl Into<Reg32>) -> Self {
        let index = index.into();
        match reg.into() {
            RegAFR::A(reg) => Reg::A(reg, index),
            RegAFR::F(reg) => Reg::F(reg, index),
            RegAFR::R(reg) => Reg::R(reg, index),
        }
    }

    /// Returns family ([`RegBlock`]) of the register
    pub fn family(self) -> RegBlock {
        match self {
            Reg::A(_, _) => RegBlock::A,
            Reg::F(_, _) => RegBlock::F,
            Reg::R(_, _) => RegBlock::R,
            Reg::S(_) => RegBlock::S,
        }
    }

    /// Returns specific register ([`RegAll`]) of the register
    pub fn register(self) -> RegAll {
        match self {
            Reg::A(reg, _) => RegAll::A(reg),
            Reg::F(reg, _) => RegAll::F(reg),
            Reg::R(reg, _) => RegAll::R(reg),
            Reg::S(_) => RegAll::S,
        }
    }

    /// Returns register index
    pub fn index(self) -> Reg32 {
        match self {
            Reg::A(_, index) | Reg::F(_, index) | Reg::R(_, index) => index,
            Reg::S(index) => index.into(),
        }
    }
}
