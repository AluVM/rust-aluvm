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

use core::fmt::{self, Debug, Formatter};

use amplify::confinement::ConfinedVec;

use super::{Site, SiteId, Status};
use crate::{Register, LIB_NAME_ALUVM};

/// Maximal size of call stack.
///
/// Equals to 0xFFFF (i.e. maximum limited by `cy` and `cp` bit size).
pub const CALL_STACK_SIZE_MAX: u16 = 0xFF;

pub trait CoreExt: Clone + Debug {
    type Reg: Register;
    type Config: Default;

    fn with(config: Self::Config) -> Self;
    fn get(&self, reg: Self::Reg) -> <Self::Reg as Register>::Value;
    fn reset(&mut self);
}

/// Registers of a single CPU/VM core.
#[derive(Clone)]
pub struct Core<
    Id: SiteId,
    Cx: CoreExt,
    const CALL_STACK_SIZE: usize = { CALL_STACK_SIZE_MAX as usize },
> {
    /// Halt register. If set to `true`, halts program when `CK` is set to [`Status::Failed`] for
    /// the first time.
    ///
    /// # See also
    ///
    /// - [`Core::ck`] register
    /// - [`Core::cf`] register
    pub(super) ch: bool,

    /// Check register, which is set on any failure (accessing register in `None` state, zero
    /// division etc.). Can be reset.
    ///
    /// # See also
    ///
    /// - [`Core::ch`] register
    /// - [`Core::cf`] register
    pub(super) ck: Status,

    /// Failure register, which counts how many times `CK` was set, and can't be reset.
    ///
    /// # See also
    ///
    /// - [`Core::ch`] register
    /// - [`Core::ck`] register
    pub(super) cf: u64,

    /// Test register, which acts as boolean test result (also a carry flag).
    pub(super) co: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is limited by 2^16 per
    /// script.
    pub(super) cy: u16,

    /// Complexity accumulator / counter.
    ///
    /// Each instruction has associated computational complexity level. This register sums
    /// complexity of executed instructions.
    ///
    /// # See also
    ///
    /// - [`Core::cy`] register
    /// - [`Core::cl`] register
    pub(super) ca: u64,

    /// Complexity limit.
    ///
    /// If this register has a value set, once [`Core::ca`] will reach this value the VM will
    /// stop program execution setting `CK` to a failure.
    pub(super) cl: Option<u64>,

    /// Call stack.
    ///
    /// # See also
    ///
    /// - [`CALL_STACK_SIZE_MAX`] constant
    /// - [`Core::cp`] register
    pub(super) cs: ConfinedVec<Site<Id>, 0, CALL_STACK_SIZE>,

    /// Core extension module.
    pub cx: Cx,
}

/// Configuration for [`Core`] initialization.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CoreConfig {
    /// Initial value for the [`Core::ch`] flag.
    pub halt: bool,
    /// Initial value for the [`Core::cl`] flag.
    pub complexity_lim: Option<u64>,
}

impl Default for CoreConfig {
    /// Sets
    /// - [`CoreConfig::halt`] to `true`,
    /// - [`CoreConfig::complexity_lim`] to `None`
    ///
    /// # See also
    ///
    /// - [`CoreConfig::halt`]
    /// - [`CoreConfig::complexity_lim`]
    /// - [`CoreConfig::field_order`]
    fn default() -> Self { CoreConfig { halt: true, complexity_lim: None } }
}

impl<Id: SiteId, Cx: CoreExt, const CALL_STACK_SIZE: usize> Core<Id, Cx, CALL_STACK_SIZE> {
    /// Initializes registers. Sets `st0` to `true`, counters to zero, call stack to empty and the
    /// rest of registers to `None` value.
    ///
    /// An alias for [`AluCore::with`]`(`[`CoreConfig::default()`]`)`.
    #[inline]
    pub fn new() -> Self {
        assert!(CALL_STACK_SIZE <= CALL_STACK_SIZE_MAX as usize, "Call stack size is too large");
        Core::with(default!(), default!())
    }

    /// Initializes registers using a configuration object [`CoreConfig`].
    pub fn with(config: CoreConfig, cx_config: Cx::Config) -> Self {
        assert!(CALL_STACK_SIZE <= CALL_STACK_SIZE_MAX as usize, "Call stack size is too large");
        Core {
            ch: config.halt,
            ck: Status::Ok,
            cf: 0,
            co: false,
            cy: 0,
            ca: 0,
            cl: config.complexity_lim,
            cs: ConfinedVec::with_capacity(CALL_STACK_SIZE),
            cx: Cx::with(cx_config),
        }
    }

    pub fn reset(&mut self) {
        let mut new = Self::new();
        new.ch = self.ch;
        new.cl = self.cl;
        new.cx.reset();
        *self = new;
    }
}

impl<Id: SiteId, Cx: CoreExt, const CALL_STACK_SIZE: usize> Debug
    for Core<Id, Cx, CALL_STACK_SIZE>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (sect, reg, val, reset) = if f.alternate() {
            ("\x1B[0;4;1m", "\x1B[0;1m", "\x1B[0;32m", "\x1B[0m")
        } else {
            ("", "", "", "")
        };

        writeln!(f, "{sect}C-regs:{reset}")?;
        write!(f, "{reg}CH{reset} {val}{}, ", self.ch)?;
        write!(f, "{reg}CK{reset} {val}{}, ", self.ck)?;
        write!(f, "{reg}CF{reset} {val}{}, ", self.cf)?;
        write!(f, "{reg}CO{reset} {val}{}, ", self.co)?;
        write!(f, "{reg}CY{reset} {val}{}, ", self.cy)?;
        write!(f, "{reg}CA{reset} {val}{}, ", self.ca)?;
        let cl = self
            .cl
            .map(|v| v.to_string())
            .unwrap_or_else(|| "~".to_string());
        write!(f, "{reg}CL{reset} {val}{cl}, ")?;
        write!(f, "{reg}CP{reset} {val}{}, ", self.cp())?;
        write!(f, "\n{reg}CS{reset} {val}")?;
        for item in &self.cs {
            write!(f, "{}   ", item)?;
        }
        writeln!(f)?;

        Debug::fmt(&self.cx, f)
    }
}
