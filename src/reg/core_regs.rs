// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2023 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023 UBIDECO Institute. All rights reserved.
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

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};

use amplify::num::apfloat::ieee;
use amplify::num::{u1024, u256, u512};
use half::bf16;

use super::{Reg32, RegA, RegAFR, RegF, RegR, RegS};
use crate::data::{ByteStr, MaybeNumber, Number};
use crate::isa::InstructionSet;
use crate::library::LibSite;

/// Maximal size of call stack.
///
/// Equals to 2^16 (limited by `cy0` and `cp0` bit size)
pub const CALL_STACK_SIZE: usize = 1 << 16;

/// Structure keeping state of all registers in a single microprosessor/VM core
#[derive(Clone)]
pub struct CoreRegs {
    // Arithmetic integer registers:
    pub(crate) a8: [Option<u8>; 32],
    pub(crate) a16: [Option<u16>; 32],
    pub(crate) a32: [Option<u32>; 32],
    pub(crate) a64: [Option<u64>; 32],
    pub(crate) a128: [Option<u128>; 32],
    pub(crate) a256: [Option<u256>; 32],
    pub(crate) a512: [Option<u512>; 32],
    pub(crate) a1024: Box<[Option<u1024>; 32]>,

    // Arithmetic float registers
    pub(crate) f16b: [Option<bf16>; 32],
    pub(crate) f16: [Option<ieee::Half>; 32],
    pub(crate) f32: [Option<ieee::Single>; 32],
    pub(crate) f64: [Option<ieee::Double>; 32],
    pub(crate) f80: [Option<ieee::X87DoubleExtended>; 32],
    pub(crate) f128: [Option<ieee::Quad>; 32],
    pub(crate) f256: [Option<ieee::Oct>; 32],
    // TODO(#5) Implement tapered floating point type
    pub(crate) f512: [Option<u512>; 32],

    // Non-arithmetic registers:
    pub(crate) r128: [Option<[u8; 16]>; 32],
    pub(crate) r160: [Option<[u8; 20]>; 32],
    pub(crate) r256: [Option<[u8; 32]>; 32],
    pub(crate) r512: [Option<[u8; 64]>; 32],
    pub(crate) r1024: Box<[Option<[u8; 128]>; 32]>,
    pub(crate) r2048: Box<[Option<[u8; 256]>; 32]>,
    pub(crate) r4096: Box<[Option<[u8; 512]>; 32]>,
    pub(crate) r8192: Box<[Option<[u8; 1024]>; 32]>,

    /// String and bytestring registers
    pub(crate) s16: Box<[Option<ByteStr>; 16]>,

    /// Control flow register which stores result of equality, comparison, boolean check and
    /// overflowing operations. Initialized with `true`.
    pub(crate) st0: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is limited by 2^16 per
    /// script.
    cy0: u16,

    /// Complexity accumulator / counter.
    ///
    /// Each instruction has associated computational complexity level. This register sums
    /// complexity of executed instructions.
    ///
    /// # See also
    ///
    /// - [`CoreRegs::cy0`] register
    /// - [`CoreRegs::cl0`] register
    ca0: u64,

    /// Complexity limit
    ///
    /// If this register has a value set, once [`CoreRegs::ca0`] will reach this value the VM will
    /// stop program execution setting `st0` to `false`.
    cl0: Option<u64>,

    /// Call stack
    ///
    /// # See also
    ///
    /// - [`CALL_STACK_SIZE`] constant
    /// - [`CoreRegs::cp0`] register
    cs0: Vec<LibSite>,

    /// Defines "top" of the call stack
    cp0: u16,
}

impl Default for CoreRegs {
    #[inline]
    fn default() -> Self {
        CoreRegs {
            a8: Default::default(),
            a16: Default::default(),
            a32: Default::default(),
            a64: Default::default(),
            a128: Default::default(),
            a256: Default::default(),
            a512: Default::default(),
            a1024: Default::default(),

            f16b: Default::default(),
            f16: Default::default(),
            f32: Default::default(),
            f64: Default::default(),
            f80: Default::default(),
            f128: Default::default(),
            f256: Default::default(),
            f512: Default::default(),

            r128: Default::default(),
            r160: Default::default(),
            r256: Default::default(),
            r512: Default::default(),
            r1024: Default::default(),
            r2048: Default::default(),
            r4096: Default::default(),
            r8192: Default::default(),

            s16: Default::default(),

            st0: true,
            cy0: 0,
            ca0: 0,
            cl0: None,
            cs0: vec![LibSite::default(); CALL_STACK_SIZE],
            cp0: 0,
        }
    }
}

impl CoreRegs {
    /// Initializes register state. Sets `st0` to `true`, counters to zero, call stack to empty and
    /// the rest of registers to `None` value.
    ///
    /// Performs exactly the same as [`CoreRegs::default()`].
    #[inline]
    pub fn new() -> CoreRegs { CoreRegs::default() }

    pub(crate) fn jmp(&mut self) -> Result<(), ()> {
        self.cy0
            .checked_add(1)
            .map(|cy| self.cy0 = cy)
            .ok_or_else(|| {
                self.st0 = false;
            })
            .map(|_| ())
    }

    pub(crate) fn call(&mut self, site: LibSite) -> Result<(), ()> {
        self.cy0
            .checked_add(1)
            .map(|cy| self.cy0 = cy)
            .ok_or_else(|| {
                self.st0 = false;
            })
            .map(|_| {
                self.cs0[self.cp0 as usize] = site;
            })
            .and_then(|_| {
                self.cp0
                    .checked_add(1)
                    .ok_or_else(|| {
                        self.st0 = false;
                    })
                    .map(|_| ())
            })
    }

    pub(crate) fn ret(&mut self) -> Option<LibSite> {
        if self.cp0 == 0 {
            None
        } else {
            self.cs0[self.cp0 as usize] = LibSite::default();
            self.cp0 -= 1;
            Some(self.cs0[self.cp0 as usize])
        }
    }

    /// Retrieves register value
    pub fn get(&self, reg: impl Into<RegAFR>, index: impl Into<Reg32>) -> MaybeNumber {
        let index = index.into() as usize;
        match reg.into() {
            RegAFR::A(a) => {
                let n = match a {
                    RegA::A8 => self.a8[index].map(Number::from),
                    RegA::A16 => self.a16[index].map(Number::from),
                    RegA::A32 => self.a32[index].map(Number::from),
                    RegA::A64 => self.a64[index].map(Number::from),
                    RegA::A128 => self.a128[index].map(Number::from),
                    RegA::A256 => self.a256[index].map(Number::from),
                    RegA::A512 => self.a512[index].map(Number::from),
                    RegA::A1024 => self.a1024[index].map(Number::from),
                };
                n.into()
            }

            RegAFR::R(r) => {
                let n = match r {
                    RegR::R128 => self.r128[index].map(Number::from),
                    RegR::R160 => self.r160[index].map(Number::from),
                    RegR::R256 => self.r256[index].map(Number::from),
                    RegR::R512 => self.r512[index].map(Number::from),
                    RegR::R1024 => self.r1024[index].map(Number::from),
                    RegR::R2048 => self.r2048[index].map(Number::from),
                    RegR::R4096 => self.r4096[index].map(Number::from),
                    RegR::R8192 => self.r8192[index].map(Number::from),
                };
                n.into()
            }

            RegAFR::F(f) => {
                let n = match f {
                    RegF::F16B => self.f16b[index].map(MaybeNumber::from),
                    RegF::F16 => self.f16[index].map(MaybeNumber::from),
                    RegF::F32 => self.f32[index].map(MaybeNumber::from),
                    RegF::F64 => self.f64[index].map(MaybeNumber::from),
                    RegF::F80 => self.f80[index].map(MaybeNumber::from),
                    RegF::F128 => self.f128[index].map(MaybeNumber::from),
                    RegF::F256 => self.f256[index].map(MaybeNumber::from),
                    RegF::F512 => self.f512[index].map(MaybeNumber::from),
                };
                n.unwrap_or_else(MaybeNumber::none)
            }
        }
    }

    /// Returns value from one of `S`-registers
    #[inline]
    pub fn get_s(&self, index: impl Into<RegS>) -> Option<&ByteStr> {
        self.s16[index.into().as_usize()].as_ref()
    }

    /// Returns value from two registers only if both of them contain a value; otherwise returns
    /// `None`.
    #[inline]
    pub fn get_both(
        &self,
        reg1: impl Into<RegAFR>,
        idx1: impl Into<Reg32>,
        reg2: impl Into<RegAFR>,
        idx2: impl Into<Reg32>,
    ) -> Option<(Number, Number)> {
        self.get(reg1, idx1).and_then(|val1| self.get(reg2, idx2).map(|val2| (val1, val2)))
    }

    /// Returns value from two string (`S`) registers only if both of them contain a value;
    /// otherwise returns `None`.
    #[inline]
    pub fn get_both_s(
        &self,
        idx1: impl Into<RegS>,
        idx2: impl Into<RegS>,
    ) -> Option<(&ByteStr, &ByteStr)> {
        self.get_s(idx1).and_then(|val1| self.get_s(idx2).map(|val2| (val1, val2)))
    }

    /// Assigns the provided value to the register bit-wise. Silently discards most significant bits
    /// until the value fits register bit size.
    ///
    /// Returns `true` if the value was not `None`
    pub fn set(
        &mut self,
        reg: impl Into<RegAFR>,
        index: impl Into<Reg32>,
        value: impl Into<MaybeNumber>,
    ) -> bool {
        let index = index.into() as usize;
        let value: Option<Number> = value.into().into();
        match reg.into() {
            RegAFR::A(a) => match a {
                RegA::A8 => self.a8[index] = value.map(Number::into),
                RegA::A16 => self.a16[index] = value.map(Number::into),
                RegA::A32 => self.a32[index] = value.map(Number::into),
                RegA::A64 => self.a64[index] = value.map(Number::into),
                RegA::A128 => self.a128[index] = value.map(Number::into),
                RegA::A256 => self.a256[index] = value.map(Number::into),
                RegA::A512 => self.a512[index] = value.map(Number::into),
                RegA::A1024 => self.a1024[index] = value.map(Number::into),
            },
            RegAFR::R(r) => match r {
                RegR::R128 => self.r128[index] = value.map(Number::into),
                RegR::R160 => self.r160[index] = value.map(Number::into),
                RegR::R256 => self.r256[index] = value.map(Number::into),
                RegR::R512 => self.r512[index] = value.map(Number::into),
                RegR::R1024 => self.r1024[index] = value.map(Number::into),
                RegR::R2048 => self.r2048[index] = value.map(Number::into),
                RegR::R4096 => self.r4096[index] = value.map(Number::into),
                RegR::R8192 => self.r8192[index] = value.map(Number::into),
            },
            RegAFR::F(f) => match f {
                RegF::F16B => self.f16b[index] = value.map(Number::into),
                RegF::F16 => self.f16[index] = value.map(Number::into),
                RegF::F32 => self.f32[index] = value.map(Number::into),
                RegF::F64 => self.f64[index] = value.map(Number::into),
                RegF::F80 => self.f80[index] = value.map(Number::into),
                RegF::F128 => self.f128[index] = value.map(Number::into),
                RegF::F256 => self.f256[index] = value.map(Number::into),
                RegF::F512 => self.f512[index] = value.map(Number::into),
            },
        }
        value.is_some()
    }

    /// Assigns the provided value to the register bit-wise if the register is not initialized.
    /// Silently discards most significant bits until the value fits register bit size.
    ///
    /// Returns `false` if the register is initialized and the value is not `None`.
    #[inline]
    pub fn set_if(
        &mut self,
        reg: impl Into<RegAFR>,
        index: impl Into<Reg32>,
        value: impl Into<MaybeNumber>,
    ) -> bool {
        let reg = reg.into();
        let index = index.into();
        if self.get(reg, index).is_none() {
            self.set(reg, index, value)
        } else {
            value.into().is_none()
        }
    }

    /// Assigns the provided value to the string register.
    ///
    /// Returns `true` if the value was not `None`.
    pub fn set_s(&mut self, index: impl Into<RegS>, value: Option<impl Into<ByteStr>>) -> bool {
        let reg = &mut self.s16[index.into().as_usize()];
        let was_set = reg.is_some();
        *reg = value.map(|v| v.into());
        was_set
    }

    /// Assigns the provided value to the string register if the register is not initialized.
    ///
    /// Returns `false` if the register is initialized and the value is not `None`.
    pub fn set_s_if(&mut self, index: impl Into<RegS>, value: Option<impl Into<ByteStr>>) -> bool {
        let index = index.into();
        if self.get_s(index).is_none() {
            self.set_s(index, value)
        } else {
            value.is_none()
        }
    }

    /// Executes provided operation (as callback function) if and only if all the provided registers
    /// contain a value (initialized). Otherwise, sets destination to `None` and does not calls the
    /// callback.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn op(
        &mut self,
        reg1: impl Into<RegAFR>,
        src1: impl Into<Reg32>,
        reg2: impl Into<RegAFR>,
        src2: impl Into<Reg32>,
        reg3: impl Into<RegAFR>,
        dst: impl Into<Reg32>,
        op: fn(Number, Number) -> Number,
    ) {
        let reg_val = match (*self.get(reg1.into(), src1), *self.get(reg2.into(), src2)) {
            (None, None) | (None, Some(_)) | (Some(_), None) => MaybeNumber::none(),
            (Some(val1), Some(val2)) => op(val1, val2).into(),
        };
        self.set(reg3.into(), dst, reg_val);
    }

    /// Accumulates complexity of the instruction into `ca0`.
    ///
    /// Sets `st0` to `false` if the complexity limit is reached or exceeded. Otherwise, does not
    /// modify `st0` value.
    ///
    /// # Returns
    ///
    /// `false` if `cl0` register has value and the accumulated complexity has reached or exceeded
    /// this limit
    #[inline]
    pub fn acc_complexity(&mut self, instr: impl InstructionSet) -> bool {
        self.ca0 = self.ca0.saturating_add(instr.complexity());
        if let Some(limit) = self.cl0 {
            if self.ca0 >= limit {
                self.st0 = false;
                false
            } else {
                true
            }
        } else {
            true
        }
    }

    /// Returns vale of `st0` register
    #[inline]
    pub fn status(&self) -> bool { self.st0 }
}

impl Debug for CoreRegs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (sect, reg, eq, val, reset) = if f.alternate() {
            ("\x1B[0;4;1m", "\x1B[0;1m", "\x1B[0;37;2m", "\x1B[0;32m", "\x1B[0m")
        } else {
            ("", "", "", "", "")
        };

        write!(f, "{}CTRL:{}\t", sect, reset)?;
        write!(f, "{}st0{}={}{} ", reg, eq, val, self.st0)?;
        write!(f, "{}cy0{}={}{} ", reg, eq, val, self.cy0)?;
        write!(f, "{}ca0{}={}{} ", reg, eq, val, self.ca0)?;
        let cl = self.cl0.map(|v| v.to_string()).unwrap_or_else(|| "~".to_string());
        write!(f, "{}cl0{}={}{} ", reg, eq, val, cl)?;
        write!(f, "{}cp0{}={}{} ", reg, eq, val, self.cp0)?;
        write!(f, "\n\t\t{}cs0{}={}", reg, eq, val)?;
        for p in 0..=self.cp0 {
            write!(f, "{}\n\t\t   ", self.cs0[p as usize])?;
        }

        write!(f, "\n{}A-REG:{}\t", sect, reset)?;
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.a8[i] {
                write!(f, "{}a8{}[{}{:02}{}]={}{:02X}{}h\t", reg, eq, reset, i, eq, val, v, reset)?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.a16[i] {
                write!(
                    f,
                    "{}a16{}[{}{:02}{}]={}{:04X}{}h\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.a32[i] {
                write!(
                    f,
                    "{}a32{}[{}{:02}{}]={}{:08X}{}h\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.a64[i] {
                write!(
                    f,
                    "{}a64{}[{}{:02}{}]={}{:016X}{}h\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        for i in 0..32 {
            if let Some(v) = self.a128[i] {
                write!(
                    f,
                    "{}a128{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.a256[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}a256{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.a512[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}a512{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.a1024[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}a1024{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }

        write!(f, "\n{}F-REG:{}\t", sect, reset)?;
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.f16b[i] {
                write!(f, "{}f16b{}[{}{:02}{}]={}{}{}\t", reg, eq, reset, i, eq, val, v, reset)?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.f16[i] {
                write!(f, "{}f16{}[{}{:02}{}]={}{}{}\t", reg, eq, reset, i, eq, val, v, reset)?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.f32[i] {
                write!(f, "{}f32{}[{}{:02}{}]={}{}{}\t", reg, eq, reset, i, eq, val, v, reset)?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        let mut c = 0;
        for i in 0..32 {
            if let Some(v) = self.f64[i] {
                write!(f, "{}f64{}[{}{:02}{}]={}{}{}\t", reg, eq, reset, i, eq, val, v, reset)?;
                c += 1;
            }
        }
        if c > 0 {
            f.write_str("\n\t\t")?;
        }
        for i in 0..32 {
            if let Some(v) = self.f80[i] {
                write!(f, "{}f80{}[{}{:02}{}]={}{}{}\n\t\t", reg, eq, reset, i, eq, val, v, reset)?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.f128[i] {
                write!(
                    f,
                    "{}f128{}[{}{:02}{}]={}{}{}\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.f256[i] {
                write!(
                    f,
                    "{}f256{}[{}{:02}{}]={}{}{}\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.f512[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}f512{}[{}{:02}{}]={}{}{}\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }

        write!(f, "\n{}R-REG:{}\t", sect, reset)?;
        for i in 0..32 {
            if let Some(v) = self.r128[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r128{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r160[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r160{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r256[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r256{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r512[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r512{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r1024[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r1024{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r2048[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r2048{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r4096[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r4096{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }
        for i in 0..32 {
            if let Some(v) = self.r8192[i] {
                let v = Number::from(v);
                write!(
                    f,
                    "{}r8192{}[{}{:02}{}]={}{:X}{}h\n\t\t",
                    reg, eq, reset, i, eq, val, v, reset
                )?;
            }
        }

        write!(f, "\n{}S-REG:{}\t", sect, reset)?;
        for i in 0..16 {
            if let Some(ref v) = self.s16[i] {
                write!(f, "{}s16{}[{}{:02}{}]={}{}{}\n\t", reg, eq, reset, i, eq, val, v, reset)?;
            }
        }
        Ok(())
    }
}
