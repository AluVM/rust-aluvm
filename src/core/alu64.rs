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

//! Microcode for ALU64 ISA.

use core::iter;

use amplify::num::{u3, u5};

use super::{AluCore, Idx32, Status};

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
}

impl From<u3> for A {
    fn from(reg: u3) -> Self {
        match reg.to_u8() {
            0 => A::A8,
            1 => A::A16,
            2 => A::A32,
            3 => A::A64,
            _ => panic!(
                "A registers above A64 are not supported under the current architecture. Consider using architecture \
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
}

impl RegA {
    pub fn with(a: A, idx: IdxA) -> Self {
        match a {
            A::A8 => Self::A8(idx),
            A::A16 => Self::A16(idx),
            A::A32 => Self::A32(idx),
            A::A64 => Self::A64(idx),
        }
    }

    pub fn bytes(self) -> u16 {
        match self {
            RegA::A8(_) => 1,
            RegA::A16(_) => 16,
            RegA::A32(_) => 32,
            RegA::A64(_) => 64,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub struct IdxA(Idx32);

impl IdxA {
    #[doc(hidden)]
    pub(crate) fn from_expected(val: usize) -> Self { Self(Idx32::from_expected(val)) }
}

impl From<u5> for IdxA {
    fn from(idx: u5) -> Self { Self(Idx32::from(idx)) }
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
impl<Id> AluCore<Id> {
    /// Returns whether check register `ck` was set to a failed state for at least once.
    pub fn ck(&self) -> Status { self.ck }

    /// Resets `ck` register.
    pub fn reset_ck(&mut self) { self.ck = Status::Ok }

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

/// Microcode for arithmetic registers.
impl<Id> AluCore<Id> {
    pub fn get(&self, reg: Reg) -> Option<u64> {
        match reg {
            Reg::A(a) => match a {
                RegA::A8(idx) => self.a8[idx.pos()].map(u64::from),
                RegA::A16(idx) => self.a16[idx.pos()].map(u64::from),
                RegA::A32(idx) => self.a32[idx.pos()].map(u64::from),
                RegA::A64(idx) => self.a64[idx.pos()],
            },
        }
    }

    pub fn a8(&self, idx: IdxA) -> Option<u8> { self.a8[idx.pos()] }
    pub fn a16(&self, idx: IdxA) -> Option<u16> { self.a16[idx.pos()] }
    pub fn a32(&self, idx: IdxA) -> Option<u32> { self.a32[idx.pos()] }
    pub fn a64(&self, idx: IdxA) -> Option<u64> { self.a64[idx.pos()] }

    pub fn clr_a8(&mut self, idx: IdxA) -> bool { self.take_a8(idx).is_some() }
    pub fn clr_a16(&mut self, idx: IdxA) -> bool { self.take_a16(idx).is_some() }
    pub fn clr_a32(&mut self, idx: IdxA) -> bool { self.take_a32(idx).is_some() }
    pub fn clr_a64(&mut self, idx: IdxA) -> bool { self.take_a64(idx).is_some() }

    pub fn take_a8(&mut self, idx: IdxA) -> Option<u8> { self.a8[idx.pos()].take() }
    pub fn take_a16(&mut self, idx: IdxA) -> Option<u16> { self.a16[idx.pos()].take() }
    pub fn take_a32(&mut self, idx: IdxA) -> Option<u32> { self.a32[idx.pos()].take() }
    pub fn take_a64(&mut self, idx: IdxA) -> Option<u64> { self.a64[idx.pos()].take() }

    pub fn set_a8(&mut self, idx: IdxA, val: u8) -> bool { self.a8[idx.pos()].replace(val).is_some() }
    pub fn set_a16(&mut self, idx: IdxA, val: u16) -> bool { self.a16[idx.pos()].replace(val).is_some() }
    pub fn set_a32(&mut self, idx: IdxA, val: u32) -> bool { self.a32[idx.pos()].replace(val).is_some() }
    pub fn set_a64(&mut self, idx: IdxA, val: u64) -> bool { self.a64[idx.pos()].replace(val).is_some() }

    pub fn swp_a8(&mut self, idx: IdxA, val: u8) -> Option<u8> { self.a8[idx.pos()].replace(val) }
    pub fn swp_a16(&mut self, idx: IdxA, val: u16) -> Option<u16> { self.a16[idx.pos()].replace(val) }
    pub fn swp_a32(&mut self, idx: IdxA, val: u32) -> Option<u32> { self.a32[idx.pos()].replace(val) }
    pub fn swp_a64(&mut self, idx: IdxA, val: u64) -> Option<u64> { self.a64[idx.pos()].replace(val) }

    pub fn a_values(&self) -> impl Iterator<Item = (RegA, u64)> + '_ {
        iter::empty()
            .chain(
                self.a8
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A8(IdxA::from_expected(i)), v as u64))),
            )
            .chain(
                self.a16
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A8(IdxA::from_expected(i)), v as u64))),
            )
            .chain(
                self.a32
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A8(IdxA::from_expected(i)), v as u64))),
            )
            .chain(
                self.a64
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A8(IdxA::from_expected(i)), v))),
            )
    }
}
