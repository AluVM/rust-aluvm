// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 LNP/BP Standards Association, Switzerland.
// Copyright (C) 2024-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2021-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use core::fmt::{self, Debug, Display, Formatter};
use core::str::FromStr;
use std::cmp::Ordering;

use crate::core::CoreExt;

pub trait Register: Copy + Ord + Debug + Display {
    type Value: Copy + Debug + Display;
    fn bytes(self) -> u16;
}

#[derive(Debug)]
pub enum NoRegs {}
impl Clone for NoRegs {
    fn clone(&self) -> Self { unreachable!() }
}
impl Copy for NoRegs {}
impl PartialEq for NoRegs {
    fn eq(&self, _: &Self) -> bool { unreachable!() }
}
impl Eq for NoRegs {}
impl Ord for NoRegs {
    fn cmp(&self, _: &Self) -> Ordering { unreachable!() }
}
impl PartialOrd for NoRegs {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> { unreachable!() }
}
impl Display for NoRegs {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result { unreachable!() }
}
impl Register for NoRegs {
    type Value = u8;
    fn bytes(self) -> u16 { unreachable!() }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
#[repr(i8)]
pub enum Status {
    #[display("ok")]
    Ok = 0,

    #[display("fail")]
    Fail = -1,
}

impl Status {
    pub fn is_ok(self) -> bool { self == Status::Ok }
}

pub trait SiteId: Copy + Ord + Debug + Display + FromStr {}

/// Location inside the instruction sequence which can be executed by the core.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Site<Id: SiteId> {
    pub prog_id: Id,
    pub offset: u16,
}

impl<Id: SiteId> Site<Id> {
    #[inline]
    pub fn new(prog_id: Id, offset: u16) -> Self { Self { prog_id, offset } }
}

impl<Id: SiteId> Display for Site<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:04X}#h", self.prog_id, self.offset)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct NoExt;

impl CoreExt for NoExt {
    type Reg = NoRegs;
    type Config = ();

    fn with(_config: Self::Config) -> Self { NoExt }

    fn get(&self, _reg: Self::Reg) -> <Self::Reg as Register>::Value { 0 }

    fn reset(&mut self) {}
}
