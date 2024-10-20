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

use core::fmt::{Debug, Display};

use amplify::confinement::TinyOrdSet;
use strict_encoding::stl::AlphaCapsNum;
use strict_encoding::RString;

use super::{ArithmInstr, CtrlInstr, Instruction, RegInstr};
use crate::stl::LIB_NAME_ALUVM;

pub const ISA_ID_MAX_LEN: usize = 16;

pub const ISA_ALU64: &str = "ALU64";
pub const ISA_AN: &str = "AN"; // Unsigned arithmetics

#[macro_export]
macro_rules! isa {
    ($id:literal) => {
        $crate::IsaId::from($id)
    };
    ($id:ident) => {
        $crate::IsaId::from($id)
    };
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, Display, FromStr)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM, dumb = { Self::from("DUMB") })]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct IsaId(RString<AlphaCapsNum, AlphaCapsNum, 1, ISA_ID_MAX_LEN>);

impl From<&'static str> for IsaId {
    fn from(id: &'static str) -> Self { Self(RString::from(id)) }
}

pub trait InstructionSet: Debug + Display {
    const ISA: &'static str;
    const ISA_EXT: &'static [&'static str];
    const HAS_EXT: bool;
    type Ext: InstructionSet;
    type Instr: Instruction;

    fn isa_id() -> IsaId { IsaId::from(Self::ISA) }

    fn isa_ext() -> TinyOrdSet<IsaId> {
        let iter = Self::ISA_EXT.into_iter().copied().map(IsaId::from);
        if Self::HAS_EXT {
            if Self::ISA != <Self::Ext as InstructionSet>::ISA {
                panic!("extension base ISA {} is not {}", <Self::Ext as InstructionSet>::ISA, Self::ISA);
            }
            TinyOrdSet::from_iter_checked(iter.chain(Self::Ext::isa_ext()))
        } else {
            TinyOrdSet::from_iter_checked(iter)
        }
    }
}

/// Reserved instruction, which equal to [`ControlFlowOp::Fail`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Default)]
#[display("halt    {0:#02X}.h")]
pub struct ReservedInstr(/** Reserved instruction op code value */ pub(super) u8);

/// Complete AluVM ISA.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum Instr<Ext: InstructionSet = ReservedInstr> {
    /// Control flow instructions.
    Ctrl(CtrlInstr),

    /// Register manipulation instructions.
    Reg(RegInstr),

    /// Arithmetic instructions for natural numbers.
    An(ArithmInstr),

    // #[cfg(feature = "str")]
    // Str(array::instr::StrInstr),
    /// Reserved instruction for future use in core `ALU` ISAs.
    Reserved(ReservedInstr),

    /// Other ISA extensions.
    Ext(Ext),
}

impl InstructionSet for ReservedInstr {
    const ISA: &'static str = ISA_ALU64;
    const ISA_EXT: &'static [&'static str] = &[];
    const HAS_EXT: bool = false;
    type Ext = Self;
    type Instr = Self;
}

impl<Ext: InstructionSet> InstructionSet for Instr<Ext> {
    const ISA: &'static str = ISA_ALU64;
    const ISA_EXT: &'static [&'static str] = &[ISA_AN];
    const HAS_EXT: bool = true;
    type Ext = ReservedInstr;
    type Instr = Self;
}
