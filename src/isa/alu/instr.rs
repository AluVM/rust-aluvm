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

use crate::core::{RegA, SiteId};
use crate::Site;

/// Value read from data segment during bytecode deserialization, which may be absent there.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
pub enum MaybeU128 {
    #[display("{0:X}#h")]
    U128(u128),

    #[display(":nodata")]
    NoData,
}

impl From<Option<u128>> for MaybeU128 {
    fn from(value: Option<u128>) -> Self {
        match value {
            None => MaybeU128::NoData,
            Some(val) => MaybeU128::U128(val),
        }
    }
}

/// Control flow instructions.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum CtrlInstr<Id: SiteId> {
    /// Not an operation.
    #[display("nop")]
    Nop,

    /// Test `CK` value, terminates if in failed state.
    #[display("chk")]
    Chk,

    /// Invert `CO` register.
    #[display("not     CO")]
    NotCo,

    /// Set `CK` register to a failed state.
    #[display("put     CK, :fail")]
    FailCk,

    /// Reset `CK` register.
    #[display("put     CK, :ok")]
    RsetCk,

    /// Jump to location (unconditionally).
    #[display("jmp     {pos:04X}#h")]
    Jmp { pos: u16 },

    /// Jump to location if `CO` is true.
    #[display("jif     CO, {pos:04X}#h")]
    JiNe { pos: u16 },

    /// Jump to location if `ck` is in a failed state.
    #[display("jif     CK, {pos:04X}#h")]
    JiFail { pos: u16 },

    /// Relative jump.
    #[display("jmp     {shift:+03X}#h")]
    Sh { shift: i8 },

    /// Relative jump if `CO` is true.
    #[display("jif     CO, {shift:+03X}#h")]
    ShNe { shift: i8 },

    /// Relative jump if `CK` is in a failed state.
    #[display("jif     CK, {shift:+03X}#h")]
    ShFail { shift: i8 },

    /// External jump.
    #[display("jmp     {site}")]
    Exec { site: Site<Id> },

    /// Subroutine call.
    #[display("call    {pos:04X}#h")]
    Fn { pos: u16 },

    /// External subroutine call.
    #[display("call    {site}")]
    Call { site: Site<Id> },

    /// Return from a subroutine or finish program.
    #[display("ret")]
    Ret,

    /// Stop the program.
    #[display("stop")]
    Stop,
}

/// Register manipulation instructions.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum RegInstr {
    /// Clear register (sets to an undefined state).
    #[display("clr     {dst}")]
    Clr { dst: RegA },

    /// Put a constant value to a register,
    #[display("put     {dst}, {val}")]
    Put { dst: RegA, val: MaybeU128 },

    /// Put a constant value to a register if it doesn't contain data,
    #[display("pif     {dst}, {val}")]
    Pif { dst: RegA, val: MaybeU128 },

    /// Test whether a register is set.
    #[display("test    {src}")]
    Test { src: RegA },

    /// Copy source to destination.
    ///
    /// If `src` and `dst` have a different bit dimension, the value is extended with zeros (as
    /// unsigned little-endian integer).
    #[display("cpy     {dst}, {src}")]
    Cpy { dst: RegA, src: RegA },

    /// Swap values of two registers.
    ///
    /// If the registers have a different bit dimension, the value of the smaller-sized register is
    /// extended with zeros (as unsigned little-endian integer) and the value of larger-sized
    /// register is divided by the modulo (the most significant bits get dropped).
    #[display("swp     {src_dst1}, {src_dst2}")]
    Swp { src_dst1: RegA, src_dst2: RegA },

    /// Check whether value of two registers is equal.
    ///
    /// If the registers have a different bit dimension, performs unsigned integer comparison using
    /// little-endian encoding.
    #[display("eq      {src1}, {src2}")]
    Eq { src1: RegA, src2: RegA },
}
