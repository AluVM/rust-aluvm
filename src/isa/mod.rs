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

//! AluVM instruction set architecture.

mod alu;
mod bytecode;
mod exec;

use alu::{ArithmInstr, BitInstr, CtrlInstr, RegInstr, SignedInstr};

use crate::library::InstructionSet;

/// List of standardised ISA extensions.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[non_exhaustive]
#[derive(Default)]
pub enum Isa {
    /// Core 64-bit ISA instruction set.
    #[display("ALU64")]
    #[default]
    Alu64,

    /// 1024-bit arithmetics and boolean logic.
    #[display("ALU1024")]
    Alu1024,

    /// Array operations with general registers.
    #[display("ARRAY")]
    Array,

    /// Instructions for SIMD.
    #[display("SIMD")]
    Simd,

    /// Floating-point operations.
    #[display("FLOAT")]
    Float,

    /// SHA hash functions.
    #[display("SHA")]
    Sha,

    /// Operations on Secp256k1 curve.
    #[display("SECP256")]
    Secp256k1,

    /// Operations on Curve25519.
    #[display("ED25519")]
    Curve25519,

    /// ALU runtime extensions.
    #[display("ALURE")]
    AluRe,

    /// Bitcoin protocol-specific instructions.
    #[display("BP")]
    Bp,

    /// RGB-specific instructions.
    #[display("RGB")]
    Rgb,

    /// Lightning network protocol-specific instructions.
    #[display("LNP")]
    Lnp,

    /// Instructions for biologically-inspired cognitive architectures.
    #[display("REBICA")]
    Rebica,
}

impl Isa {
    /// Enumerates all ISA extension variants.
    pub const fn all() -> [Isa; 13] {
        [
            Isa::Alu64,
            Isa::Alu1024,
            Isa::Float,
            Isa::Array,
            Isa::Simd,
            Isa::Sha,
            Isa::Secp256k1,
            Isa::Curve25519,
            Isa::AluRe,
            Isa::Bp,
            Isa::Rgb,
            Isa::Lnp,
            Isa::Rebica,
        ]
    }
}

/// Reserved instruction, which equal to [`ControlFlowOp::Fail`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Default)]
#[display("rsrv    {0:#04X}")]
pub struct ReservedInstr(/** Reserved instruction op code value */ pub(super) u8);

/// Complete AluVM ISA.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum Instr<Extension = ReservedInstr>
where Extension: InstructionSet
{
    /// Control flow instructions.
    Ctrl(CtrlInstr),

    /// Register manipulation instructions.
    Reg(RegInstr),

    /// Arithmetic instructions for natural numbers.
    An(ArithmInstr),

    /// Sign-aware arithmetic instructions.
    Az(SignedInstr),

    /// Bit-manipulation and boolean arithmetic instructions.
    Bit(BitInstr),

    /// Floating-point arithmetic instructions.
    #[cfg(feature = "float")]
    Float(alu::FloatInstr),

    /// Array register (`r`) instructions.
    #[cfg(feature = "array")]
    Array(alu::ArrayInstr),

    /// Bytestring register (`s`) instructions.
    #[cfg(feature = "str")]
    Str(alu::StrInstr),

    /// Reserved instruction for future use in core `ALU` ISA.
    Reserved(ReservedInstr),

    /// Other ISA extensions.
    Ext(Extension),
}
