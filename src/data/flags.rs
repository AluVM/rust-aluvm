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

use amplify::num::apfloat::Round;

/// Encoding and overflowing flags for integer numbers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct IntFlags {
    /// Treat the integer as signed (`true`) or unsigned (`false`). Signed integers has a different
    /// behaviour on detecting overflows, since they use only 7 bits for significant digits and not
    /// 8.
    pub signed: bool,

    /// With addition / subtraction / multiplication, indicates whether overflow must result in
    /// modulo-based wrapping (`true`) or set the destination into `None` state (`false`).
    /// With division, `true` means that Euclidean division should be performed.
    pub wrap: bool,
}

impl IntFlags {
    /// Constructs variant for unsigned checked operation flags
    #[inline]
    pub fn unsigned_checked() -> Self {
        IntFlags {
            signed: false,
            wrap: false,
        }
    }

    /// Constructs variant for signed checked operation flags
    #[inline]
    pub fn signed_checked() -> Self {
        IntFlags {
            signed: true,
            wrap: false,
        }
    }

    /// Constructs variant for unsigned wrapped operation flags
    #[inline]
    pub fn unsigned_wrapped() -> Self {
        IntFlags {
            signed: false,
            wrap: true,
        }
    }

    /// Constructs variant for signed wrapped operation flags
    #[inline]
    pub fn signed_wrapped() -> Self {
        IntFlags {
            signed: true,
            wrap: true,
        }
    }
}

/// Rounding flags for float numbers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum RoundingFlag {
    /// Round always toward zero, which means ceiling for negative numbers and flooring for
    /// positive numbers.
    TowardsZero = 0,

    /// Round to the nearest neighbour, and if the number is exactly in the middle, ties round to
    /// the nearest even digit in the required position.
    #[default]
    TowardsNearest = 1,

    /// Round down (flooring), ie toward -∞; negative results thus round away from zero.
    Floor = 2,

    /// Round up (ceiling), ie toward +∞; negative results thus round toward zero.
    Ceil = 3,
}

impl From<RoundingFlag> for Round {
    fn from(flag: RoundingFlag) -> Self {
        match flag {
            RoundingFlag::TowardsZero => Round::TowardZero,
            RoundingFlag::TowardsNearest => Round::NearestTiesToEven,
            RoundingFlag::Floor => Round::TowardNegative,
            RoundingFlag::Ceil => Round::TowardPositive,
        }
    }
}
