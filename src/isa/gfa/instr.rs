// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 UBIDECO Labs,
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

use crate::core::{IdxAl, RegA, A};

/// Arithmetic instructions for finite fields.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum FieldInstr {
    /// Increment register value using finite-field (modulo) arithmetics of the `order`.
    #[display("incmod  {src_dst}, {val}")]
    IncMod {
        /// Destination register.
        src_dst: RegA,
        /// Value to add.
        val: u8,
    },

    /// Decrement register value using finite-field (modulo) arithmetics of the `order`.
    #[display("decmod  {src_dst}, {val}")]
    DecMod {
        /// Destination register.
        src_dst: RegA,
        /// Value to add.
        val: u8,
    },

    /// Negate value using finite-field arithmetics.
    #[display("negmod  {src_dst}")]
    NegMod { src_dst: RegA },

    /// Add `src` value to `src_dst` value using finite-field (modulo) arithmetics of the `order`.
    #[display("addmod  {reg}{dst}, {reg}{src1}, {reg}{src2}")]
    AddMod {
        reg: A,
        dst: IdxAl,
        src1: IdxAl,
        src2: IdxAl,
    },

    /// Multiply `src` value to `src_dst` value using finite-field (modulo) arithmetics of the
    /// `order`.
    #[display("mulmod  {reg}{dst}, {reg}{src1}, {reg}{src2}")]
    MulMod {
        reg: A,
        dst: IdxAl,
        src1: IdxAl,
        src2: IdxAl,
    },
}
