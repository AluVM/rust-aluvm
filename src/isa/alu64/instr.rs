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

use amplify::num::u2;

use crate::core::IdxA;
use crate::regs::{RegA, A};

/// Control flow instructions.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum CtrlInstr {
    /// Not an operation.
    Nop,

    /// Test ck value, terminates if in failed state.
    Chk,

    /// Invert `ct` register.
    NotCk,

    /// Set `ck` register to a failed state.
    Fail,

    /// Reset `ck` register.
    Rset,

    /// Jump to location (unconditionally).
    Jmp,

    /// Jump to location if `ct` is true.
    Jif,

    /// Jump to location if `ck` is in a failed state.
    JiFail,

    /// Relative jump.
    Sh,

    /// Relative jump if `ct` is true.
    ShIf,

    /// Relative jump if `ck` is in a failed state.
    ShIfail,

    /// External jump.
    Exec,

    /// Subroutine call.
    Fn,

    /// External subroutine call.
    Call,

    /// Return from a subroutine or finish program.
    Ret,

    /// Stop the program.
    Stop,
}

/// Register manipulation instructions.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum RegInstr {
    /// Clear register (sets to an undefined state).
    Clr { dst: A },

    /// Put a constant value to a register,
    Put { dst: A, val: u64 },

    /// Put a constant value to a register if it doesn't contain data,
    Pif { dst: A, val: u64 },

    /// Test whether a register is set.
    Test { src: A },

    /// Copy source to destination.
    ///
    /// If `src` and `dst` have a different bit dimension, the value is extended with zeros (as
    /// unsigned little-endian integer).
    Cpy { dst: A, src: A },

    /// Swap values of two registers.
    ///
    /// If the registers have a different bit dimension, the value of the smaller-sized register is
    /// extended with zeros (as unsigned little-endian integer) and the value of larger-sized
    /// register is divided by the modulo (the most significant bits get dropped).
    Swp { src_dst1: A, src_dst2: A },

    /// Check whether value of two registers is equal.
    ///
    /// If the registers have a different bit dimension, performs unsigned integer comparison using
    /// little-endian encoding.
    Eq { src1: A, src2: A },
}

/// Arithmetic instructions for finite fields.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum FieldInstr {
    /// Increment register value using finite-field (modulo) arithmetics of the `order`.
    IncMod {
        /// Destination register.
        dst: RegA,
        /// Value to add.
        val: u2,
        /// Order of the finite field.
        // 2-bit smaller than complete no of bytes.
        order: u64,
    },

    /// Decrement register value using finite-field (modulo) arithmetics of the `order`.
    DecMod {
        /// Destination register.
        dst: RegA,
        /// Value to add.
        val: u2,
        /// Order of the finite field.
        // 2-bit smaller than complete no of bytes.
        order: u64,
    },

    /// Add `src` value to `src_dst` value using finite-field (modulo) arithmetics of the `order`.
    AddMod {
        src_dst: RegA,
        src: IdxA,
        /// Order of the finite field.
        // 2-bit smaller than complete no of bytes.
        order: u64,
    },

    /// Negate value using finite-field arithmetics.
    NegMod {
        dst: RegA,
        src: IdxA,
        /// Order of the finite field.
        // 2-bit smaller than complete no of bytes.
        order: u64,
    },

    /// Multiply `src` value to `src_dst` value using finite-field (modulo) arithmetics of the
    /// `order`.
    MulMod {
        src_dst: RegA,
        src: IdxA,
        /// Order of the finite field.
        // 2-bit smaller than complete no of bytes.
        order: u64,
    },
}
