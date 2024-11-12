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

use crate::core::{Core, CoreExt, SiteId, Status};
use crate::Site;

/// Microcode for flag registers.
impl<Id: SiteId, Cx: CoreExt, const CALL_STACK_SIZE: usize> Core<Id, Cx, CALL_STACK_SIZE> {
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
