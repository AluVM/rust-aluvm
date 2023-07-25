// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023 by
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

use core::cmp::Ordering;

use amplify::num::apfloat::ieee;
use half::bf16;

use super::{FloatLayout, Layout, Number};
use crate::data::ByteArray;

impl PartialEq for ByteArray {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.to_clean()[..].eq(&other.to_clean()[..])
    }
}

impl Eq for ByteArray {}

impl PartialOrd for ByteArray {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ByteArray {
    fn cmp(&self, _other: &Self) -> Ordering { todo!() }
}

impl PartialEq for Number {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.layout() == other.layout()
            || (self.layout().is_signed_int() && other.layout().is_unsigned_int())
            || (self.layout().is_unsigned_int() && other.layout().is_signed_int()))
            && self.to_clean()[..].eq(&other.to_clean()[..])
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

/// Since we always convert `NaN` values into `None` and keep them at the level of `MaybeNumber`, we
/// can do strict ordering even on float numbers
impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.layout(), other.layout(), "comparing numbers with different layout");
        match self.layout() {
            Layout::Integer(_) => match (self.is_positive(), other.is_positive()) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => self.to_u1024_bytes().cmp(&other.to_u1024_bytes()),
            },
            Layout::Float(FloatLayout::BFloat16) => {
                bf16::from(self).partial_cmp(&bf16::from(other)).expect("number value contains NaN")
            }
            Layout::Float(FloatLayout::IeeeHalf) => ieee::Half::from(self)
                .partial_cmp(&ieee::Half::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeSingle) => ieee::Single::from(self)
                .partial_cmp(&ieee::Single::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeDouble) => ieee::Double::from(self)
                .partial_cmp(&ieee::Double::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::X87DoubleExt) => ieee::X87DoubleExtended::from(self)
                .partial_cmp(&ieee::X87DoubleExtended::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeQuad) => ieee::Quad::from(self)
                .partial_cmp(&ieee::Quad::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeOct) => {
                unimplemented!("IEEE-754 256-bit floats are not yet supported")
            }
            Layout::Float(FloatLayout::FloatTapered) => {
                unimplemented!("512-bit tapered floats are not yet supported")
            }
        }
    }
}

impl Number {
    /// Does comparison by ignoring the difference in the last bit of significand for float layouts.
    /// For integers performs normal comparison.
    pub fn rounding_cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.layout(), other.layout(), "comparing numbers with different layout");
        match self.layout() {
            Layout::Integer(_) => self.cmp(other),
            Layout::Float(FloatLayout::FloatTapered) => {
                unimplemented!("512-bit tapered floats are not yet supported")
            }
            Layout::Float(float_layout) => {
                let last_bit = Number::masked_bit(
                    float_layout
                        .significand_pos()
                        .expect("non-tapered float layout does not provides significand position")
                        .end,
                    self.layout(),
                );
                (*self ^ last_bit).cmp(&(*other ^ last_bit))
            }
        }
    }

    /// Checks for the equality ignoring the difference in the last bit of significand for float
    /// layouts. For integers performs normal comparison.
    #[inline]
    pub fn rounding_eq(&self, other: &Self) -> bool { self.rounding_cmp(other) == Ordering::Equal }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_numbers() {
        let x = Number::from(0);
        let y = Number::from(0);
        assert_eq!(x, y);
        let x = Number::from(0);
        let y = Number::from(1);
        assert!(x < y);
        let x = Number::from(1);
        let y = Number::from(-1);
        assert!(x > y);
        let x = Number::from(-128i8);
        let y = Number::from(-127i8);
        assert!(x < y);
    }
}
