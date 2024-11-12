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

use crate::core::SiteId;
use crate::Site;

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
