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

use crate::core::SiteId;
use crate::regs::A;
use crate::Site;

/// Control flow instructions.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
pub enum CtrlInstr<Id: SiteId> {
    /// Not an operation.
    #[display("nop")]
    Nop,

    /// Test ck value, terminates if in failed state.
    #[display("chk")]
    Chk,

    /// Invert `co` register.
    #[display("not     co")]
    NotCo,

    /// Set `ck` register to a failed state.
    #[display("put     ck, fail")]
    FailCk,

    /// Reset `ck` register.
    #[display("put     ck, ok")]
    OkCk,

    /// Jump to location (unconditionally).
    #[display("jmp     {pos:04x}.h")]
    Jmp { pos: u16 },

    /// Jump to location if `co` is true.
    #[display("jif     co, {pos:04x}.h")]
    JifCo { pos: u16 },

    /// Jump to location if `ck` is in a failed state.
    #[display("jif     ck, {pos:04x}.h")]
    JifCk { pos: u16 },

    /// Relative jump.
    #[display("jmp     {pos:+03x}.h")]
    Shift { pos: i8 },

    /// Relative jump if `co` is true.
    #[display("jif     co, {pos:+03x}.h")]
    ShIfCo { pos: i8 },

    /// Relative jump if `ck` is in a failed state.
    #[display("jif     ck, {pos:+03x}.h")]
    ShIfCk { pos: i8 },

    /// External jump.
    #[display("jmp     {site}")]
    Exec { site: Site<Id> },

    /// Subroutine call.
    #[display("call    {pos:04x}.h")]
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
    Clr { dst: A },

    /// Put a constant value to a register,
    #[display("put     {dst}, {val:x}.h")]
    Put { dst: A, val: u64 },

    /// Put a constant value to a register if it doesn't contain data,
    #[display("pif     {dst}, {val:x}.h")]
    Pif { dst: A, val: u64 },

    /// Test whether a register is set.
    #[display("test    {src}")]
    Test { src: A },

    /// Copy source to destination.
    ///
    /// If `src` and `dst` have a different bit dimension, the value is extended with zeros (as
    /// unsigned little-endian integer).
    #[display("cpy     {dst}, {src}")]
    Cpy { dst: A, src: A },

    /// Swap values of two registers.
    ///
    /// If the registers have a different bit dimension, the value of the smaller-sized register is
    /// extended with zeros (as unsigned little-endian integer) and the value of larger-sized
    /// register is divided by the modulo (the most significant bits get dropped).
    #[display("swp     {src_dst1}, {src_dst2}")]
    Swp { src_dst1: A, src_dst2: A },

    /// Check whether value of two registers is equal.
    ///
    /// If the registers have a different bit dimension, performs unsigned integer comparison using
    /// little-endian encoding.
    #[display("eq      {src1}, {src2}")]
    Eq { src1: A, src2: A },
}
