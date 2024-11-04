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

//! Microcode for ALU64 ISA.

use core::iter;

use crate::core::{Core, IdxA, Reg, RegA, SiteId};

/// Microcode for arithmetic registers.
impl<Id: SiteId, const CALL_STACK_SIZE: usize> Core<Id, CALL_STACK_SIZE> {
    pub fn get(&self, reg: Reg) -> Option<u128> {
        match reg {
            Reg::A(a) => match a {
                RegA::A8(idx) => self.a8[idx.pos()].map(u128::from),
                RegA::A16(idx) => self.a16[idx.pos()].map(u128::from),
                RegA::A32(idx) => self.a32[idx.pos()].map(u128::from),
                RegA::A64(idx) => self.a64[idx.pos()].map(u128::from),
                RegA::A128(idx) => self.a128[idx.pos()],
            },
        }
    }

    pub fn a(&self, reg: RegA) -> Option<u128> {
        match reg {
            RegA::A8(idx) => self.a8(idx).map(u128::from),
            RegA::A16(idx) => self.a16(idx).map(u128::from),
            RegA::A32(idx) => self.a32(idx).map(u128::from),
            RegA::A64(idx) => self.a64(idx).map(u128::from),
            RegA::A128(idx) => self.a128(idx),
        }
    }

    pub fn clr_a(&mut self, reg: RegA) -> bool {
        match reg {
            RegA::A8(idx) => self.clr_a8(idx),
            RegA::A16(idx) => self.clr_a16(idx),
            RegA::A32(idx) => self.clr_a32(idx),
            RegA::A64(idx) => self.clr_a64(idx),
            RegA::A128(idx) => self.clr_a128(idx),
        }
    }

    pub fn set_a(&mut self, reg: RegA, val: u128) -> bool {
        match reg {
            RegA::A8(idx) => self.set_a8(idx, val as u8),
            RegA::A16(idx) => self.set_a16(idx, val as u16),
            RegA::A32(idx) => self.set_a32(idx, val as u32),
            RegA::A64(idx) => self.set_a64(idx, val as u64),
            RegA::A128(idx) => self.set_a128(idx, val),
        }
    }

    pub fn take_a(&mut self, reg: RegA) -> Option<u128> {
        match reg {
            RegA::A8(idx) => self.take_a8(idx).map(u128::from),
            RegA::A16(idx) => self.take_a16(idx).map(u128::from),
            RegA::A32(idx) => self.take_a32(idx).map(u128::from),
            RegA::A64(idx) => self.take_a64(idx).map(u128::from),
            RegA::A128(idx) => self.take_a128(idx),
        }
    }

    pub fn swp_a(&mut self, reg: RegA, val: u128) -> Option<u128> {
        match reg {
            RegA::A8(idx) => self.swp_a8(idx, val as u8).map(u128::from),
            RegA::A16(idx) => self.swp_a16(idx, val as u16).map(u128::from),
            RegA::A32(idx) => self.swp_a32(idx, val as u32).map(u128::from),
            RegA::A64(idx) => self.swp_a64(idx, val as u64).map(u128::from),
            RegA::A128(idx) => self.swp_a128(idx, val),
        }
    }

    pub fn a8(&self, idx: IdxA) -> Option<u8> { self.a8[idx.pos()] }
    pub fn a16(&self, idx: IdxA) -> Option<u16> { self.a16[idx.pos()] }
    pub fn a32(&self, idx: IdxA) -> Option<u32> { self.a32[idx.pos()] }
    pub fn a64(&self, idx: IdxA) -> Option<u64> { self.a64[idx.pos()] }
    pub fn a128(&self, idx: IdxA) -> Option<u128> { self.a128[idx.pos()] }

    pub fn clr_a8(&mut self, idx: IdxA) -> bool { self.take_a8(idx).is_some() }
    pub fn clr_a16(&mut self, idx: IdxA) -> bool { self.take_a16(idx).is_some() }
    pub fn clr_a32(&mut self, idx: IdxA) -> bool { self.take_a32(idx).is_some() }
    pub fn clr_a64(&mut self, idx: IdxA) -> bool { self.take_a64(idx).is_some() }
    pub fn clr_a128(&mut self, idx: IdxA) -> bool { self.take_a128(idx).is_some() }

    pub fn take_a8(&mut self, idx: IdxA) -> Option<u8> { self.a8[idx.pos()].take() }
    pub fn take_a16(&mut self, idx: IdxA) -> Option<u16> { self.a16[idx.pos()].take() }
    pub fn take_a32(&mut self, idx: IdxA) -> Option<u32> { self.a32[idx.pos()].take() }
    pub fn take_a64(&mut self, idx: IdxA) -> Option<u64> { self.a64[idx.pos()].take() }
    pub fn take_a128(&mut self, idx: IdxA) -> Option<u128> { self.a128[idx.pos()].take() }

    pub fn set_a8(&mut self, idx: IdxA, val: u8) -> bool { self.a8[idx.pos()].replace(val).is_some() }
    pub fn set_a16(&mut self, idx: IdxA, val: u16) -> bool { self.a16[idx.pos()].replace(val).is_some() }
    pub fn set_a32(&mut self, idx: IdxA, val: u32) -> bool { self.a32[idx.pos()].replace(val).is_some() }
    pub fn set_a64(&mut self, idx: IdxA, val: u64) -> bool { self.a64[idx.pos()].replace(val).is_some() }
    pub fn set_a128(&mut self, idx: IdxA, val: u128) -> bool { self.a128[idx.pos()].replace(val).is_some() }

    pub fn swp_a8(&mut self, idx: IdxA, val: u8) -> Option<u8> { self.a8[idx.pos()].replace(val) }
    pub fn swp_a16(&mut self, idx: IdxA, val: u16) -> Option<u16> { self.a16[idx.pos()].replace(val) }
    pub fn swp_a32(&mut self, idx: IdxA, val: u32) -> Option<u32> { self.a32[idx.pos()].replace(val) }
    pub fn swp_a64(&mut self, idx: IdxA, val: u64) -> Option<u64> { self.a64[idx.pos()].replace(val) }
    pub fn swp_a128(&mut self, idx: IdxA, val: u128) -> Option<u128> { self.a128[idx.pos()].replace(val) }

    pub fn a_values(&self) -> impl Iterator<Item = (RegA, u128)> + '_ {
        iter::empty()
            .chain(
                self.a8
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A8(IdxA::from_expected(i)), v as u128))),
            )
            .chain(
                self.a16
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A16(IdxA::from_expected(i)), v as u128))),
            )
            .chain(
                self.a32
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A32(IdxA::from_expected(i)), v as u128))),
            )
            .chain(
                self.a64
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A64(IdxA::from_expected(i)), v as u128))),
            )
            .chain(
                self.a128
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (RegA::A128(IdxA::from_expected(i)), v))),
            )
    }
}
