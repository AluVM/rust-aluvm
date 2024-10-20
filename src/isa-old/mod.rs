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

//! AluVM instruction set architecture

#[macro_use]
mod macros;
mod bytecode;
mod exec;
mod flags;
mod instr;
pub mod opcodes;

pub use bytecode::{Bytecode, BytecodeError};
pub use exec::{ExecStep, InstructionSet};
pub use flags::{
    DeleteFlag, ExtendFlag, Flag, FloatEqFlag, InsertFlag, IntFlags, MergeFlag, NoneEqFlag,
    ParseFlagError, RoundingFlag, SignFlag, SplitFlag,
};
pub use instr::{
    ArithmeticOp, BitwiseOp, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp, Instr, MoveOp,
    PutOp, ReservedOp, Secp256k1Op,
};

/// List of standardised ISA extensions.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[non_exhaustive]
#[derive(Default)]
pub enum Isa {
    /// Core ISA instruction set
    #[display("ALU")]
    #[default]
    Alu,

    /// Floating-point operations
    #[display("FLOAT")]
    Float,

    /// Bitcoin-specific cryptographic hash functions
    #[display("BPDIGEST")]
    BpDigest,

    /// Operations on Secp256k1 curve
    #[display("SECP256")]
    Secp256k1,

    /// Operations on Curve25519
    #[display("ED25519")]
    Curve25519,

    /// ALU runtime extensions
    #[display("ALURE")]
    AluRe,

    /// Bitcoin protocol-specific instructions
    #[display("BP")]
    Bp,

    /// RGB-specific instructions
    #[display("RGB")]
    Rgb,

    /// Lightning network protocol-specific instructions
    #[display("LNP")]
    Lnp,

    /// Instructions for SIMD
    #[display("SIMD")]
    Simd,

    /// Instructions for biologically-inspired cognitive architectures
    #[display("REBICA")]
    Rebica,
}

impl Isa {
    /// Enumerates all ISA extension variants
    pub const fn all() -> [Isa; 11] {
        [
            Isa::Alu,
            Isa::Float,
            Isa::BpDigest,
            Isa::Secp256k1,
            Isa::Curve25519,
            Isa::AluRe,
            Isa::Bp,
            Isa::Rgb,
            Isa::Lnp,
            Isa::Simd,
            Isa::Rebica,
        ]
    }
}
