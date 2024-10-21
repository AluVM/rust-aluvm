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

use core::fmt::{self, Debug, Formatter};

use super::{Site, SiteId, Status};
#[cfg(feature = "GFA")]
use crate::core::gfa::Zp;

/// Maximal size of call stack.
///
/// Equals to 0xFFFF (i.e. maximum limited by `cy` and `cp` bit size).
pub const CALL_STACK_SIZE_MAX: u16 = 0xFF;

/// Registers of a single CPU/VM core.
#[derive(Clone)]
pub struct Core<Id: SiteId> {
    #[cfg(feature = "GFA")]
    /// Finite field order.
    pub(super) zp: Zp,

    // ============================================================================================
    // Arithmetic integer registers (ALU64 ISA).
    pub(super) a8: [Option<u8>; 32],
    pub(super) a16: [Option<u16>; 32],
    pub(super) a32: [Option<u32>; 32],
    pub(super) a64: [Option<u64>; 32],
    pub(super) a128: [Option<u128>; 32],

    // ============================================================================================
    // Arithmetic integer registers (A1024 ISA extension).

    //pub(super) a128: [Option<u128>; 32],
    //pub(super) a256: [Option<u256>; 32],
    //pub(super) a512: [Option<u512>; 32],
    //pub(super) a1024: Box<[Option<u1024>; 32]>,

    // ============================================================================================
    // Arithmetic float registers (`FLOAT` ISA extension).

    //pub(super) f16b: [Option<bf16>; 32],
    //pub(super) f16: [Option<ieee::Half>; 32],
    //pub(super) f32: [Option<ieee::Single>; 32],
    //pub(super) f64: [Option<ieee::Double>; 32],
    //pub(super) f80: [Option<ieee::X87DoubleExtended>; 32],
    //pub(super) f128: [Option<ieee::Quad>; 32],
    //pub(super) f256: [Option<ieee::Oct>; 32],
    // TODO(#5) Implement tapered floating point type
    //pub(super) f512: [Option<u512>; 32],

    // ============================================================================================
    // Array registers (`ARRAY` ISA extension).

    //pub(super) r128: [Option<[u8; 16]>; 32],
    //pub(super) r160: [Option<[u8; 20]>; 32],
    //pub(super) r256: [Option<[u8; 32]>; 32],
    //pub(super) r512: [Option<[u8; 64]>; 32],
    //pub(super) r1024: [Option<Box<[u8; 128]>>; 32],
    //pub(super) r2048: [Option<Box<[u8; 256]>>; 32],
    //pub(super) r4096: [Option<Box<[u8; 512]>>; 32],
    //pub(super) r8192: [Option<Box<[u8; 1024]>>; 32],

    // ============================================================================================
    // /// Bytestring registers (`STR` ISA extension).
    //#[cfg(feature = "str")]
    //pub(super) b: [Option<Box<[ByteStr; 16]>>],

    // --------------------------------------------------------------------------------------------
    // Control flow registers
    /// Halt register. If set to `true`, halts program when `ck` is set to `true` for the first
    /// time.
    ///
    /// # See also
    ///
    /// - [`Core::ck`] register
    /// - [`Core::cf`] register
    ch: bool,

    /// Check register, which is set on any failure (accessing register in `None` state, zero
    /// division etc.). Can be reset.
    ///
    /// # See also
    ///
    /// - [`Core::ch`] register
    /// - [`Core::cf`] register
    pub(super) ck: Status,

    /// Failure register, which is set on the first time `ck` is set, and can't be reset.
    ///
    /// # See also
    ///
    /// - [`Core::ch`] register
    /// - [`Core::ck`] register
    cf: Status,

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
    /// stop program execution setting `ck` to `false`.
    cl: Option<u64>,

    /// Call stack.
    ///
    /// # See also
    ///
    /// - [`CALL_STACK_SIZE_MAX`] constant
    /// - [`Core::cp`] register
    pub(super) cs: Vec<Site<Id>>,

    /// Defines "top" of the call stack.
    pub(super) cp: u16,
}

/// Configuration for [`Core`] initialization.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CoreConfig {
    /// Initial value for the [`Core::ch`] flag.
    pub halt: bool,
    /// Initial value for the [`Core::cl`] flag.
    pub complexity_lim: Option<u64>,
    /// Size of the call stack in the [`Core::cs`] register.
    pub call_stack_size: u16,
    #[cfg(feature = "GFA")]
    /// Order of the finite field for modulo arithmetics.
    pub field_order: Zp,
}

impl Default for CoreConfig {
    /// Sets
    /// - [`CoreConfig::halt`] to `true`,
    /// - [`CoreConfig::complexity_lim`] to `None`
    /// - [`CoreConfig::call_stack_size`] to [`CALL_STACK_SIZE_MAX`],
    /// - [`CoreConfig::field_order`] to [`Zp::F1137119`] (if `GFA` feature is set).
    ///
    /// # See also
    ///
    /// - [`CoreConfig::halt`]
    /// - [`CoreConfig::complexity_lim`]
    /// - [`CoreConfig::call_stack_size`]
    /// - [`CoreConfig::field_order`]
    fn default() -> Self {
        CoreConfig {
            halt: true,
            complexity_lim: None,
            call_stack_size: CALL_STACK_SIZE_MAX,
            #[cfg(feature = "GFA")]
            field_order: Zp::F1137119,
        }
    }
}

impl<Id: SiteId> Core<Id> {
    /// Initializes registers. Sets `st0` to `true`, counters to zero, call stack to empty and the
    /// rest of registers to `None` value.
    ///
    /// An alias for [`AluCore::with`]`(`[`CoreConfig::default()`]`)`.
    #[inline]
    pub fn new() -> Self { Core::with(default!()) }

    /// Initializes registers using a configuration object [`CoreConfig`].
    pub fn with(config: CoreConfig) -> Self {
        Core {
            #[cfg(feature = "GFA")]
            zp: config.field_order,
            a8: Default::default(),
            a16: Default::default(),
            a32: Default::default(),
            a64: Default::default(),
            a128: Default::default(),

            //#[cfg(feature = "str")]
            //b: Default::default(),
            ch: config.halt,
            ck: Status::Ok,
            cf: Status::Ok,
            co: false,
            cy: 0,
            ca: 0,
            cl: config.complexity_lim,
            cs: Vec::with_capacity(config.call_stack_size as usize),
            cp: 0,
        }
    }
}

/// Microcode for flag registers.
impl<Id: SiteId> Core<Id> {
    /// Return whether check register `ck` was set to a failed state for at least once.
    pub fn had_failed(&self) -> bool { self.cf == Status::Fail }

    /// Return complexity limit value.
    pub fn cl(&self) -> Option<u64> { return self.cl; }
}

impl<Id: SiteId> Debug for Core<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (sect, reg, val, reset) =
            if f.alternate() { ("\x1B[0;4;1m", "\x1B[0;1m", "\x1B[0;32m", "\x1B[0m") } else { ("", "", "", "") };

        writeln!(f, "{sect}C-regs:{reset}")?;
        write!(f, "{reg}ch{reset} {val}{}, ", self.ch)?;
        write!(f, "{reg}ck{reset} {val}{}, ", self.ck)?;
        write!(f, "{reg}cf{reset} {val}{}, ", self.cf)?;
        write!(f, "{reg}ct{reset} {val}{}, ", self.co)?;
        write!(f, "{reg}cy{reset} {val}{}, ", self.cy)?;
        write!(f, "{reg}ca{reset} {val}{}, ", self.ca)?;
        let cl = self
            .cl
            .map(|v| v.to_string())
            .unwrap_or_else(|| "~".to_string());
        write!(f, "{reg}cl{reset} {val}{cl}, ")?;
        write!(f, "{reg}cp{reset} {val}{}, ", self.cp)?;
        write!(f, "\n{reg}cs{reset} {val}")?;
        for p in 0..=self.cp {
            write!(f, "{}   ", self.cs[p as usize])?;
        }
        writeln!(f)?;

        writeln!(f, "{sect}A-regs:{reset}")?;
        let mut c = 0;
        for (i, v) in self.a_values() {
            writeln!(f, "{reg}{i}{reset} {val}{v:X}{reset}h")?;
            c += 1;
        }
        if c > 0 {
            writeln!(f)?;
        }

        /*
        #[cfg(feature = "str")]
        {
            writeln!(f, "{sect}B-regs:{reset}")?;
            for (i, v) in self.b_values() {
                writeln!(f, "{reg}{i}{reset} {val}{v}{reset}")?;
            }
        }
         */

        Ok(())
    }
}
