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

use core::fmt::Debug;

use amplify::num::{u3, u4, u5};

use crate::core::{Core, Idx16, Idx32, SiteId, Status};
use crate::Site;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
pub enum A {
    #[display("A8")]
    A8,
    #[display("A16")]
    A16,
    #[display("A32")]
    A32,
    #[display("A64")]
    A64,
    #[display("A128")]
    A128,
}

impl A {
    pub fn to_u3(&self) -> u3 {
        match self {
            A::A8 => u3::with(0),
            A::A16 => u3::with(1),
            A::A32 => u3::with(2),
            A::A64 => u3::with(3),
            A::A128 => u3::with(4),
        }
    }
}

impl From<u3> for A {
    fn from(reg: u3) -> Self {
        match reg.to_u8() {
            0 => A::A8,
            1 => A::A16,
            2 => A::A32,
            3 => A::A64,
            4 => A::A128,
            _ => panic!(
                "A registers above A128 are not supported under the current architecture. Consider using architecture \
                 extension."
            ),
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
pub enum RegA {
    #[display("A8{0}")]
    A8(IdxA),
    #[display("A16{0}")]
    A16(IdxA),
    #[display("A32{0}")]
    A32(IdxA),
    #[display("A64{0}")]
    A64(IdxA),
    #[display("A128{0}")]
    A128(IdxA),
}

impl RegA {
    pub fn with(a: A, idx: IdxA) -> Self {
        match a {
            A::A8 => Self::A8(idx),
            A::A16 => Self::A16(idx),
            A::A32 => Self::A32(idx),
            A::A64 => Self::A64(idx),
            A::A128 => Self::A128(idx),
        }
    }

    pub fn bytes(self) -> u16 {
        match self {
            RegA::A8(_) => 1,
            RegA::A16(_) => 16,
            RegA::A32(_) => 32,
            RegA::A64(_) => 64,
            RegA::A128(_) => 128,
        }
    }

    pub fn a(self) -> A {
        match self {
            RegA::A8(_) => A::A8,
            RegA::A16(_) => A::A16,
            RegA::A32(_) => A::A32,
            RegA::A64(_) => A::A64,
            RegA::A128(_) => A::A128,
        }
    }

    pub fn idx(self) -> IdxA {
        match self {
            RegA::A8(idx) => idx,
            RegA::A16(idx) => idx,
            RegA::A32(idx) => idx,
            RegA::A64(idx) => idx,
            RegA::A128(idx) => idx,
        }
    }

    pub fn to_u8(&self) -> u8 {
        let a = self.a().to_u3().to_u8();
        let idx = self.idx().to_u5().to_u8();
        a << 5 + idx
    }
}

impl From<u8> for RegA {
    fn from(val: u8) -> Self {
        let a = u3::with(val >> 5);
        let idx = u5::with(val & 0x1F);
        RegA::with(A::from(a), IdxA::from(idx))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub struct IdxA(Idx32);

impl IdxA {
    #[doc(hidden)]
    pub(crate) fn from_expected(val: usize) -> Self { Self(Idx32::from_expected(val)) }

    pub fn to_u5(&self) -> u5 { u5::with(self.0 as u8) }
}

impl From<u5> for IdxA {
    fn from(idx: u5) -> Self { Self(Idx32::from(idx)) }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub struct IdxAl(Idx16);

impl IdxAl {
    #[allow(dead_code)]
    #[doc(hidden)]
    pub(crate) fn from_expected(val: usize) -> Self { Self(Idx16::from_expected(val)) }

    pub fn to_u4(&self) -> u4 { u4::with(self.0 as u8) }
}

impl From<IdxAl> for IdxA {
    fn from(idx: IdxAl) -> IdxA { IdxA::from_expected(idx.0 as usize) }
}

impl From<u4> for IdxAl {
    fn from(idx: u4) -> Self { Self(Idx16::from(idx)) }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(inner)]
pub enum Reg {
    #[from]
    A(RegA),
}

impl Reg {
    pub fn bytes(self) -> u16 {
        match self {
            Reg::A(a) => a.bytes(),
        }
    }
}

impl IdxA {
    #[inline]
    pub fn pos(&self) -> usize { self.0 as usize }
}

/// Microcode for flag registers.
impl<Id: SiteId, const CALL_STACK_SIZE: usize> Core<Id, CALL_STACK_SIZE> {
    /// Read overflow/carry flag.
    pub fn co(&self) -> bool { self.co }

    /// Set overflow/carry flag to a value.
    pub fn set_co(&mut self, co: bool) { self.co = co; }

    /// Return how many times `ck` was set to a failed state.
    pub fn cf(&self) -> u64 { self.cf }

    /// Return `true` if `ck` was in a failed state for at least once.
    pub fn has_failed(&self) -> bool { self.cf > 0 }

    /// Return whether check register `ck` is in a failed state.
    pub fn ck(&self) -> Status { self.ck }

    /// Set `CK` register to a failed state.
    ///
    /// Returns whether further execution should be stopped (i.e. `ch` register value).
    #[must_use]
    pub fn fail_ck(&mut self) -> bool {
        self.ck = Status::Fail;
        self.cf += 1;
        self.ch
    }

    /// Reset `CK` register.
    pub fn reset_ck(&mut self) { self.ck = Status::Ok }

    /// Return size of the call stack.
    pub fn cp(&self) -> u16 { self.cs.len() as u16 }

    /// Push a location to a call stack.
    ///
    /// # Returns
    ///
    /// Top of the call stack.
    pub fn push_cs(&mut self, from: Site<Id>) -> Option<u16> {
        self.cs.push(from).ok()?;
        Some(self.cp())
    }

    /// Pops a call stack item.
    pub fn pop_cs(&mut self) -> Option<Site<Id>> { self.cs.pop() }

    /// Return complexity limit value.
    pub fn cl(&self) -> Option<u64> { self.cl }

    /// Accumulate complexity value.
    ///
    /// # Returns
    ///
    /// Boolean indicating wheather complexity limit is reached.
    pub fn acc_complexity(&mut self, complexity: u64) -> bool {
        self.ca = self.ca.saturating_add(complexity);
        self.cl().map(|lim| self.ca >= lim).unwrap_or_default()
    }
}
