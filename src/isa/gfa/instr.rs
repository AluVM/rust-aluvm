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

use crate::core::{IdxA, RegA};

const M31: u128 = (1 << 31u128) - 1;
const F1137119: u128 = 1 + 11 * 37 * (1 << 119u128);
const F1289: u128 = u128::MAX - 8; // it should be 9, but `u128::MAX` is 2^128-1 and not 2^128

/// Finite field orders.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
pub enum Zp {
    #[display("M31", alt = "2^31-1")]
    M31, // 2^31-1
    #[display("F1137119", alt = "1+11*37*2^119")]
    F1137119,
    #[display("F1289", alt = "2^128-9")]
    F1289,
    #[display("{0:x}.h")]
    Other(u128),
}

impl From<Zp> for u128 {
    fn from(zp: Zp) -> Self { zp.to_u128() }
}

impl Zp {
    pub fn to_u128(self) -> u128 {
        match self {
            Zp::M31 => M31,
            Zp::F1137119 => F1137119,
            Zp::F1289 => F1289,
            Zp::Other(val) => val,
        }
    }
}

/// Arithmetic instructions for finite fields.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum FieldInstr {
    /// Increment register value using finite-field (modulo) arithmetics of the `order`.
    #[display("incmod  {src_dst}, {val}, {order}")]
    IncMod {
        /// Destination register.
        src_dst: RegA,
        /// Value to add.
        val: u8,
        /// Order of the finite field.
        order: Zp,
    },

    /// Decrement register value using finite-field (modulo) arithmetics of the `order`.
    #[display("decmod  {src_dst}, {val}, {order}")]
    DecMod {
        /// Destination register.
        src_dst: RegA,
        /// Value to add.
        val: u8,
        /// Order of the finite field.
        order: Zp,
    },

    /// Add `src` value to `src_dst` value using finite-field (modulo) arithmetics of the `order`.
    #[display("addmod  {src_dst}, {src}, {order}")]
    AddMod {
        src_dst: RegA,
        src: IdxA,
        /// Order of the finite field.
        order: Zp,
    },

    /// Negate value using finite-field arithmetics.
    #[display("negmod  {dst}, {src}, {order}")]
    NegMod {
        dst: RegA,
        src: IdxA,
        /// Order of the finite field.
        order: Zp,
    },

    /// Multiply `src` value to `src_dst` value using finite-field (modulo) arithmetics of the
    /// `order`.
    #[display("mulmod  {src_dst}, {src}, {order}")]
    MulMod {
        src_dst: RegA,
        src: IdxA,
        /// Order of the finite field.
        order: Zp,
    },
}
