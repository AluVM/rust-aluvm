// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Labs,
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

use amplify::num::u3;

use crate::data as number;
use crate::reg::Register;

/// Common set of methods handled by different sets and families of VM registers
pub trait NumericRegister: Register {
    /// Register bit dimension
    #[inline]
    fn bits(&self) -> u16 { self.bytes() * 8 }

    /// Returns register layout
    fn layout(&self) -> number::Layout;
}

/// Enumeration of integer arithmetic registers (`A`-registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
#[derive(Default)]
pub enum RegA {
    /// 8-bit arithmetics register
    #[display("a8")]
    A8 = 0,

    /// 16-bit arithmetics register
    #[display("a16")]
    A16 = 1,

    /// 32-bit arithmetics register
    #[display("a32")]
    A32 = 2,

    /// 64-bit arithmetics register
    #[display("a64")]
    #[default]
    A64 = 3,

    /// 128-bit arithmetics register
    #[display("a128")]
    A128 = 4,

    /// 256-bit arithmetics register
    #[display("a256")]
    A256 = 5,

    /// 512-bit arithmetics register
    #[display("a512")]
    A512 = 6,

    /// 1024-bit arithmetics register
    #[display("a1024")]
    A1024 = 7,
}

impl Register for RegA {
    #[inline]
    fn description() -> &'static str { "arithmetic register" }

    #[inline]
    fn bytes(self) -> u16 {
        match self {
            RegA::A8 => 1,
            RegA::A16 => 2,
            RegA::A32 => 4,
            RegA::A64 => 8,
            RegA::A128 => 16,
            RegA::A256 => 32,
            RegA::A512 => 64,
            RegA::A1024 => 128,
        }
    }
}

impl NumericRegister for RegA {
    #[inline]
    fn layout(&self) -> number::Layout { number::Layout::unsigned(self.bytes()) }
}

impl RegA {
    /// Set of all A registers
    pub const ALL: [RegA; 8] = [
        RegA::A8,
        RegA::A16,
        RegA::A32,
        RegA::A64,
        RegA::A128,
        RegA::A256,
        RegA::A512,
        RegA::A1024,
    ];

    /// Constructs [`RegA`] object for a provided requirement for register bit size
    pub fn with(bits: u16) -> Option<Self> {
        Some(match bits {
            8 => RegA::A8,
            16 => RegA::A16,
            32 => RegA::A32,
            64 => RegA::A64,
            128 => RegA::A128,
            256 => RegA::A256,
            512 => RegA::A512,
            1024 => RegA::A1024,
            _ => return None,
        })
    }

    /// Returns integer layout [`number::IntLayout`] specific for this register
    #[inline]
    pub fn int_layout(self) -> number::IntLayout { number::IntLayout::unsigned(self.bytes()) }
}

impl From<&RegA> for u3 {
    fn from(rega: &RegA) -> Self { u3::with(*rega as u8) }
}

impl From<RegA> for u3 {
    fn from(rega: RegA) -> Self { u3::with(rega as u8) }
}

impl From<u3> for RegA {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegA::A8.into() => RegA::A8,
            v if v == RegA::A16.into() => RegA::A16,
            v if v == RegA::A32.into() => RegA::A32,
            v if v == RegA::A64.into() => RegA::A64,
            v if v == RegA::A128.into() => RegA::A128,
            v if v == RegA::A256.into() => RegA::A256,
            v if v == RegA::A512.into() => RegA::A512,
            v if v == RegA::A1024.into() => RegA::A1024,
            _ => unreachable!(),
        }
    }
}

/// Enumeration of float arithmetic registers (`F`-registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
#[derive(Default)]
pub enum RegF {
    /// 16-bit bfloat16 format used in machine learning
    #[display("f16b")]
    F16B = 0,

    /// 16-bit IEEE-754 binary16 half-precision
    #[display("f16")]
    F16 = 1,

    /// 32-bit IEEE-754 binary32 single-precision
    #[display("f32")]
    F32 = 2,

    /// 64-bit IEEE-754 binary64 double-precision
    #[display("f64")]
    #[default]
    F64 = 3,

    /// 80-bit IEEE-754 extended precision
    #[display("f80")]
    F80 = 4,

    /// 128-bit IEEE-754 binary128 quadruple precision
    #[display("f128")]
    F128 = 5,

    /// 256-bit IEEE-754 binary256 octuple precision
    #[display("f256")]
    F256 = 6,

    /// 512-bit tapered floating point
    #[display("f512")]
    F512 = 7,
}

impl Register for RegF {
    #[inline]
    fn description() -> &'static str { "floating-point register" }

    #[inline]
    fn bytes(self) -> u16 {
        match self {
            RegF::F16B => 2,
            RegF::F16 => 2,
            RegF::F32 => 4,
            RegF::F64 => 8,
            RegF::F80 => 10,
            RegF::F128 => 16,
            RegF::F256 => 32,
            RegF::F512 => 64,
        }
    }
}

impl NumericRegister for RegF {
    #[inline]
    fn layout(&self) -> number::Layout {
        let fl = match self {
            RegF::F16B => number::FloatLayout::BFloat16,
            RegF::F16 => number::FloatLayout::IeeeHalf,
            RegF::F32 => number::FloatLayout::IeeeSingle,
            RegF::F64 => number::FloatLayout::IeeeDouble,
            RegF::F80 => number::FloatLayout::X87DoubleExt,
            RegF::F128 => number::FloatLayout::IeeeQuad,
            RegF::F256 => number::FloatLayout::IeeeOct,
            RegF::F512 => number::FloatLayout::FloatTapered,
        };
        number::Layout::float(fl)
    }
}

impl RegF {
    /// Set of all F registers
    pub const ALL: [RegF; 8] = [
        RegF::F16B,
        RegF::F16,
        RegF::F32,
        RegF::F64,
        RegF::F80,
        RegF::F128,
        RegF::F256,
        RegF::F512,
    ];

    /// Constructs [`RegF`] object for a provided requirement for register bit size
    pub fn with(bits: u16, use_bfloat16: bool) -> Option<Self> {
        Some(match bits {
            16 => {
                if use_bfloat16 {
                    RegF::F16B
                } else {
                    RegF::F16
                }
            }
            32 => RegF::F32,
            64 => RegF::F64,
            80 => RegF::F80,
            128 => RegF::F128,
            256 => RegF::F256,
            512 => RegF::F512,
            _ => return None,
        })
    }
}

impl From<&RegF> for u3 {
    fn from(regf: &RegF) -> Self { u3::with(*regf as u8) }
}

impl From<RegF> for u3 {
    fn from(regf: RegF) -> Self { u3::with(regf as u8) }
}

impl From<u3> for RegF {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegF::F16B.into() => RegF::F16B,
            v if v == RegF::F16.into() => RegF::F16,
            v if v == RegF::F32.into() => RegF::F32,
            v if v == RegF::F64.into() => RegF::F64,
            v if v == RegF::F80.into() => RegF::F80,
            v if v == RegF::F128.into() => RegF::F128,
            v if v == RegF::F256.into() => RegF::F256,
            v if v == RegF::F512.into() => RegF::F512,
            _ => unreachable!(),
        }
    }
}

/// Enumeration of the set of general registers (`R`-registers: non-arithmetic registers, mostly
/// used for cryptography)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
#[derive(Default)]
pub enum RegR {
    /// 128-bit non-arithmetics register
    #[display("r128")]
    R128 = 0,

    /// 160-bit non-arithmetics register
    #[display("r160")]
    R160 = 1,

    /// 256-bit non-arithmetics register
    #[display("r256")]
    #[default]
    R256 = 2,

    /// 512-bit non-arithmetics register
    #[display("r512")]
    R512 = 3,

    /// 1024-bit non-arithmetics register
    #[display("r1024")]
    R1024 = 4,

    /// 2048-bit non-arithmetics register
    #[display("r2048")]
    R2048 = 5,

    /// 4096-bit non-arithmetics register
    #[display("r4096")]
    R4096 = 6,

    /// 8192-bit non-arithmetics register
    #[display("r8192")]
    R8192 = 7,
}

impl Register for RegR {
    #[inline]
    fn description() -> &'static str { "array register" }

    #[inline]
    fn bytes(self) -> u16 {
        match self {
            RegR::R128 => 16,
            RegR::R160 => 20,
            RegR::R256 => 32,
            RegR::R512 => 64,
            RegR::R1024 => 128,
            RegR::R2048 => 256,
            RegR::R4096 => 512,
            RegR::R8192 => 1024,
        }
    }
}

impl NumericRegister for RegR {
    #[inline]
    fn layout(&self) -> number::Layout { number::Layout::unsigned(self.bytes()) }
}

impl RegR {
    /// Set of all R registers
    pub const ALL: [RegR; 8] = [
        RegR::R128,
        RegR::R160,
        RegR::R256,
        RegR::R512,
        RegR::R1024,
        RegR::R2048,
        RegR::R4096,
        RegR::R8192,
    ];

    /// Constructs [`RegR`] object for a provided requirement for register bit size
    #[inline]
    pub fn with(bits: u16) -> Option<Self> {
        Some(match bits {
            128 => RegR::R128,
            160 => RegR::R160,
            256 => RegR::R256,
            512 => RegR::R512,
            1024 => RegR::R1024,
            2048 => RegR::R2048,
            4096 => RegR::R4096,
            8192 => RegR::R8192,
            _ => return None,
        })
    }
}

impl From<&RegR> for u3 {
    fn from(regr: &RegR) -> Self { u3::with(*regr as u8) }
}

impl From<RegR> for u3 {
    fn from(regr: RegR) -> Self { u3::with(regr as u8) }
}

impl From<u3> for RegR {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegR::R128.into() => RegR::R128,
            v if v == RegR::R160.into() => RegR::R160,
            v if v == RegR::R256.into() => RegR::R256,
            v if v == RegR::R512.into() => RegR::R512,
            v if v == RegR::R1024.into() => RegR::R1024,
            v if v == RegR::R2048.into() => RegR::R2048,
            v if v == RegR::R4096.into() => RegR::R4096,
            v if v == RegR::R8192.into() => RegR::R8192,
            _ => unreachable!(),
        }
    }
}
