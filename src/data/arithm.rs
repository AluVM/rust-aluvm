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

use core::cmp::Ordering;
use core::ops::{Neg, Rem};

use amplify::num::apfloat::{ieee, Float};
use half::bf16;

use super::{FloatLayout, IntFlags, IntLayout, Layout, Number, NumberLayout, RoundingFlag};
use crate::data::MaybeNumber;

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

impl Number {
    /// Addition of two integers with configuration flags for overflow and signed format.
    /// If `signed` flag is inconsistent with Number layout,
    /// the layout will be discarded before computing.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn int_add(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "adding numbers with different layout");
        match (layout, flags.signed) {
            (Layout::Integer(IntLayout { bytes, .. }), true) => self
                .to_i1024_bytes()
                .checked_add(rhs.to_i1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::signed(n.layout().bytes()), true))
                .and_then(|mut n| (n.reshape(Layout::signed(bytes)) || flags.wrap).then(|| n)),
            (Layout::Integer(IntLayout { bytes, .. }), false) => self
                .to_u1024_bytes()
                .checked_add(rhs.to_u1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::unsigned(n.layout().bytes()), true))
                .and_then(|mut n| (n.reshape(Layout::unsigned(bytes)) || flags.wrap).then(|| n)),
            (Layout::Float(_), _) => panic!("integer addition of float numbers"),
        }
    }

    /// Subtraction of two integers with configuration flags for overflow and signed format.
    /// If `signed` flag is inconsistent with Number layout,
    /// the layout will be discarded before computing.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn int_sub(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "subtracting numbers with different layout");
        match (layout, flags.signed) {
            (Layout::Integer(IntLayout { bytes, .. }), true) => self
                .to_i1024_bytes()
                .checked_sub(rhs.to_i1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::signed(n.layout().bytes()), true))
                .and_then(|mut n| (n.reshape(Layout::signed(bytes)) || flags.wrap).then(|| n)),
            (Layout::Integer(IntLayout { bytes, .. }), false) => self
                .to_u1024_bytes()
                .checked_sub(rhs.to_u1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::unsigned(n.layout().bytes()), true))
                .and_then(|mut n| (n.reshape(Layout::unsigned(bytes)) || flags.wrap).then(|| n)),
            (Layout::Float(_), _) => panic!("integer subtraction of float numbers"),
        }
    }

    /// Multiplication of two integers with configuration flags for overflow and signed format.
    /// If `signed` flag is inconsistent with Number layout,
    /// the layout will be discarded before computing.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn int_mul(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "multiplying numbers with different layout");
        match (layout, flags.signed) {
            (Layout::Integer(IntLayout { bytes, .. }), true) => self
                .to_i1024_bytes()
                .checked_mul(rhs.to_i1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::signed(n.layout().bytes()), true))
                .and_then(|mut n| (n.reshape(Layout::signed(bytes)) || flags.wrap).then(|| n)),
            (Layout::Integer(IntLayout { bytes, .. }), false) => self
                .to_u1024_bytes()
                .checked_mul(rhs.to_u1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::unsigned(n.layout().bytes()), true))
                .and_then(|mut n| (n.reshape(Layout::unsigned(bytes)) || flags.wrap).then(|| n)),
            (Layout::Float(_), _) => panic!("integer multiplication of float numbers"),
        }
    }

    /// Division of two integers with configuration flags for Euclidean division and signed format.
    /// If `signed` flag is inconsistent with Number layout,
    /// the layout will be discarded before computing.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn int_div(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "dividing numbers with different layout");

        if rhs.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some(Number::zero(layout));
        }

        match (layout, flags.signed) {
            (Layout::Integer(IntLayout { bytes, .. }), true) => {
                let res = match flags.wrap {
                    true => self.to_i1024_bytes().checked_div_euclid(rhs.to_i1024_bytes()),
                    false => self.to_i1024_bytes().checked_div(rhs.to_i1024_bytes()),
                };
                res.map(Number::from)
                    .and_then(|n| n.reshaped(Layout::signed(n.layout().bytes()), true))
                    .and_then(|n| n.reshaped(Layout::signed(bytes), false))
            }
            (Layout::Integer(IntLayout { bytes, .. }), false) => self
                .to_u1024_bytes()
                .checked_div(rhs.to_u1024_bytes())
                .map(Number::from)
                .and_then(|n| n.reshaped(Layout::signed(bytes), false)),
            (Layout::Float(_), _) => panic!("integer division of float numbers"),
        }
    }

    /// Addition of two floats with configuration flags for rounding.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn float_add(self, rhs: Self, flag: RoundingFlag) -> MaybeNumber {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "adding numbers with different layout");
        match layout {
            Layout::Float(FloatLayout::BFloat16) => (bf16::from(self) + bf16::from(rhs)).into(),
            Layout::Float(FloatLayout::IeeeHalf) => {
                ieee::Half::from(self).add_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                ieee::Single::from(self).add_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                ieee::Double::from(self).add_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                ieee::Quad::from(self).add_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                ieee::X87DoubleExtended::from(self).add_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeOct) => {
                ieee::Oct::from(self).add_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::FloatTapered) => todo!("(#5) tapered float addition"),
            Layout::Integer(_) => panic!("float addition of integer numbers"),
        }
    }

    /// Subtraction of two floats with configuration flags for rounding.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn float_sub(self, rhs: Self, flag: RoundingFlag) -> MaybeNumber {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "subtracting numbers with different layout");
        match layout {
            Layout::Float(FloatLayout::BFloat16) => (bf16::from(self) - bf16::from(rhs)).into(),
            Layout::Float(FloatLayout::IeeeHalf) => {
                ieee::Half::from(self).sub_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                ieee::Single::from(self).sub_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                ieee::Double::from(self).sub_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                ieee::Quad::from(self).sub_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                ieee::X87DoubleExtended::from(self).sub_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeOct) => {
                ieee::Oct::from(self).sub_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::FloatTapered) => todo!("(#5) tapered float subtraction"),
            Layout::Integer(_) => panic!("float subtraction of integer numbers"),
        }
    }

    /// Multiplication of two floats with configuration flags for rounding.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn float_mul(self, rhs: Self, flag: RoundingFlag) -> MaybeNumber {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "multiplying numbers with different layout");
        match layout {
            Layout::Float(FloatLayout::BFloat16) => (bf16::from(self) * bf16::from(rhs)).into(),
            Layout::Float(FloatLayout::IeeeHalf) => {
                ieee::Half::from(self).mul_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                ieee::Single::from(self).mul_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                ieee::Double::from(self).mul_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                ieee::Quad::from(self).mul_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                ieee::X87DoubleExtended::from(self).mul_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeOct) => {
                ieee::Oct::from(self).mul_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::FloatTapered) => todo!("(#5) tapered float multiplication"),
            Layout::Integer(_) => panic!("float multiplication of integer numbers"),
        }
    }

    /// Division of two floats with configuration flags for rounding.
    ///
    /// # Panics
    ///
    /// - if applied to float number layouts
    /// - if numbers in arguments has different layout.
    pub fn float_div(self, rhs: Self, flag: RoundingFlag) -> MaybeNumber {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "dividing numbers with different layout");
        match layout {
            Layout::Float(FloatLayout::BFloat16) => (bf16::from(self) / bf16::from(rhs)).into(),
            Layout::Float(FloatLayout::IeeeHalf) => {
                ieee::Half::from(self).div_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                ieee::Single::from(self).div_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                ieee::Double::from(self).div_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                ieee::Quad::from(self).div_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                ieee::X87DoubleExtended::from(self).div_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::IeeeOct) => {
                ieee::Oct::from(self).div_r(rhs.into(), flag.into()).into()
            }
            Layout::Float(FloatLayout::FloatTapered) => todo!("(#5) tapered float division"),
            Layout::Integer(_) => panic!("float division of integer numbers"),
        }
    }

    /// Adds or removes negative sign to the number (negates negative or positive number, depending
    /// on the method argument value)
    ///
    /// # Returns
    /// Result of the operation as an optional - or `None` if the operation was impossible,
    /// specifically:
    ///  - applied to unsigned integer layout
    ///  - an attempt to negate the minimum possible value for its layout (e.g. -128 as 1 byte)
    #[inline]
    pub fn applying_sign(mut self, sign: impl Into<bool>) -> Option<Number> {
        let layout = self.layout();
        match layout {
            Layout::Integer(IntLayout { signed: true, .. }) => {
                if !self.is_positive() && self.count_ones() == 1 {
                    // attempt to negate the minimum possible value for its layout
                    None
                } else if !self.is_positive() ^ sign.into() {
                    let mut one = Number::from(1u8);
                    one.reshape(layout);
                    (!self).int_add(one, IntFlags {
                        signed: true,
                        wrap: true,
                    })
                } else {
                    Some(self)
                }
            }
            Layout::Integer(IntLayout { signed: false, .. }) => {
                // applied to unsigned integer layout
                None
            }
            Layout::Float(..) => {
                let sign_byte = layout.sign_byte();
                if sign.into() {
                    self[sign_byte] |= 0x80;
                } else {
                    self[sign_byte] &= 0x7F;
                }
                Some(self)
            }
        }
    }

    /// Removes negative sign if present (negates negative number)
    #[inline]
    pub fn without_sign(self) -> Option<Number> { self.applying_sign(false) }
}

impl Rem for Number {
    type Output = Option<Number>;

    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            return None;
        }
        let layout = self.layout();
        Some(match layout {
            Layout::Integer(IntLayout { signed: true, .. }) => {
                let val1 = self.to_u1024_bytes();
                let val2 = rhs.to_u1024_bytes();
                val1.rem(val2).into()
            }
            Layout::Integer(IntLayout { signed: false, .. }) if layout.bits() <= 128 => {
                let val1 = i128::from(self);
                let val2 = i128::from(rhs);
                val1.rem(val2).into()
            }
            Layout::Integer(IntLayout { .. }) => {
                todo!("(#11) implement large signed number modulo division algorithm")
            }
            Layout::Float(_) => panic!("modulo division of float number"),
        })
    }
}

impl Neg for Number {
    type Output = Option<Number>;

    fn neg(self) -> Self::Output { self.applying_sign(self.is_positive()) }
}

impl Number {
    /// Returns the absolute value of the number
    pub fn abs(self) -> Option<Number> {
        if self.is_positive() {
            Some(self)
        } else {
            self.applying_sign(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

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

    #[test]
    fn int_add() {
        let x = Number::from(1);
        let y = Number::from(2);
        let z = Number::from(3);
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: false,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(255u8);
        let y = Number::from(1u8);
        let z = Number::from(0u8);
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: false,
                wrap: true
            }),
            Some(z)
        );
        let x = Number::from(1i8);
        let y = Number::from(-1i8);
        let z = Number::from(0i8);
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: true,
                wrap: true
            }),
            Some(z)
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
        let x = Number::from(-2i8);
        let y = Number::from(-1i8);
        let z = Number::from(-3i8);
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: true,
                wrap: true
            }),
            Some(z)
        );
        assert_eq!(
            x.int_add(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
    }

    #[test]
    fn int_sub() {
        let x = Number::from(3u8);
        let y = Number::from(2u8);
        let z = Number::from(1u8);
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: false,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(0i8);
        let y = Number::from(42i8);
        let z = Number::from(-42i8);
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
        let x = Number::from(6000);
        let y = Number::from(5000);
        let z = Number::from(1000);
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: false,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(-10i8);
        let y = Number::from(-4i8);
        let z = Number::from(-6i8);
        let w = Number::from(6u8);
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
        // 246 - 252
        assert_eq!(
            x.int_sub(y, IntFlags {
                signed: false,
                wrap: true
            }),
            None
        );
        assert_eq!(
            y.int_sub(x, IntFlags {
                signed: false,
                wrap: true
            }),
            Some(w)
        );
    }

    #[test]
    fn int_mul() {
        let x = Number::from(2);
        let y = Number::from(3);
        let z = Number::from(6);
        assert_eq!(
            x.int_mul(y, IntFlags {
                signed: false,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(128u8);
        let y = Number::from(2u8);
        let z = Number::from(0u8);
        assert_eq!(
            x.int_mul(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
        assert_eq!(
            x.int_mul(y, IntFlags {
                signed: false,
                wrap: true
            }),
            Some(z)
        );
        let x = Number::from(4);
        let y = Number::from(0);
        let z = Number::from(0);
        assert_eq!(
            x.int_mul(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(-2);
        let y = Number::from(-5);
        let z = Number::from(10);
        assert_eq!(
            x.int_mul(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_mul(y, IntFlags {
                signed: true,
                wrap: true
            }),
            Some(z)
        );
    }

    #[test]
    fn int_div() {
        let x = Number::from(6);
        let y = Number::from(3);
        let z = Number::from(2);
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: false,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(7);
        let y = Number::from(2);
        let z = Number::from(3);
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: false,
                wrap: false
            }),
            Some(z)
        );
        let x = Number::from(4u8);
        let y = Number::from(0u8);
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: false,
                wrap: false
            }),
            None
        );
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: false,
                wrap: true
            }),
            None
        );
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: true,
                wrap: false
            }),
            None
        );
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: true,
                wrap: true
            }),
            None
        );
        let x = Number::from(-7i8);
        let y = Number::from(4i8);
        let z = Number::from(-1i8);
        let w = Number::from(-2i8);
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: true,
                wrap: false
            }),
            Some(z)
        );
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: true,
                wrap: true
            }),
            Some(w)
        );
        let x = Number::from(-128i8);
        let y = Number::from(-1i8);
        let z = Number::from(0i8);
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: true,
                wrap: false
            }),
            None
        );
        assert_eq!(
            x.int_div(y, IntFlags {
                signed: false,
                wrap: true
            }),
            Some(z)
        );
    }

    #[test]
    fn applying_sign() {
        let x = Number::from(1i8);
        let y = Number::from(-1i8);
        assert_eq!(x.applying_sign(true).unwrap(), y);
        assert_eq!(x, y.applying_sign(false).unwrap());

        let x = Number::from(1i8);
        let y = Number::from(1i8);
        assert_eq!(x.applying_sign(false).unwrap(), y);

        let x = MaybeNumber::from(bf16::from_f32(7.0_f32)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(-7.0_f32)).unwrap();
        assert_ne!(x, y);
        assert_eq!(x.applying_sign(true).unwrap(), y);

        let x = MaybeNumber::from(bf16::from_f32(7.0_f32)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(7.0_f32)).unwrap();
        assert_eq!(x, y);
        assert_eq!(x.applying_sign(false).unwrap(), y);
    }

    #[test]
    fn applying_sign_to_unsigned() {
        let x = Number::from(1u8);
        assert_eq!(None, x.applying_sign(false));
    }

    #[test]
    fn applying_sign_to_minimum() {
        let x = Number::from(-128i8);
        assert_eq!(None, x.applying_sign(false));
    }

    #[test]
    fn without_sign() {
        let x = Number::from(1i8);
        let y = Number::from(-1i8);
        assert_eq!(x.without_sign().unwrap(), x);
        assert_eq!(y.without_sign().unwrap(), x);
    }

    #[test]
    fn neg() {
        let x = Number::from(1i8);
        let y = Number::from(-1i8);
        assert_ne!(x, y);
        assert_eq!(x.neg().unwrap(), y);
        assert_eq!(x, y.neg().unwrap());

        let x = MaybeNumber::from(bf16::from_f32(7.0_f32)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(-7.0_f32)).unwrap();
        assert_ne!(x, y);
        assert_eq!(x.neg().unwrap(), y);
        assert_eq!(x, y.neg().unwrap());
    }

    #[test]
    fn abs() {
        let x = Number::from(1i8);
        let y = Number::from(-1i8);
        assert_eq!(x, y.abs().unwrap());

        let x = MaybeNumber::from(bf16::from_f32(12.3_f32)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(-12.3_f32)).unwrap();
        assert_eq!(x, y.abs().unwrap());

        let x = Number::from(-128i8);
        assert_eq!(None, x.abs());

        let x = Number::from(-128i16);
        let y = Number::from(128i16);
        assert_eq!(x.abs().unwrap(), y);
    }

    #[test]
    fn float_add() {
        let x = MaybeNumber::from(ieee::Single::from_str("0x1p+0").unwrap()).unwrap();
        let y = MaybeNumber::from(ieee::Single::from_str("0x1p+0").unwrap()).unwrap();
        let z = MaybeNumber::from(ieee::Single::from_str("0x1p+1").unwrap());
        assert_eq!(x.float_add(y, RoundingFlag::Ceil), z);

        let x = MaybeNumber::from(ieee::Single::from_str("0x1p+0").unwrap()).unwrap();
        let y = MaybeNumber::from(ieee::Single::ZERO);
        assert_eq!(x.float_add((-x).unwrap(), RoundingFlag::Ceil), y);

        // overflow
        let x = MaybeNumber::from(ieee::Single::largest()).unwrap();
        let y = MaybeNumber::from(ieee::Single::from_str("0x1p+0").unwrap()).unwrap();
        assert_eq!(x.float_add(y, RoundingFlag::Ceil), MaybeNumber::none());
    }

    #[test]
    fn float_sub() {
        let x = MaybeNumber::from(ieee::Oct::from_str("0x1p+1").unwrap()).unwrap();
        let y = MaybeNumber::from(ieee::Oct::from_str("0x1p+0").unwrap()).unwrap();
        let z = MaybeNumber::from(ieee::Oct::from_str("0x1p+0").unwrap());
        assert_eq!(x.float_sub(y, RoundingFlag::Ceil), z);

        // INF - (-INF) = INF
        let x = MaybeNumber::from(ieee::Single::INFINITY).unwrap();
        assert_eq!(x.float_sub((-x).unwrap(), RoundingFlag::Ceil), MaybeNumber::from(x));

        // INF - INF = NaN
        let x = MaybeNumber::from(ieee::Single::INFINITY).unwrap();
        assert_eq!(x.float_sub(x, RoundingFlag::Ceil), MaybeNumber::none());
    }

    #[test]
    fn float_mul() {
        let x = MaybeNumber::from(ieee::Single::from_str("0x1p+1").unwrap()).unwrap();
        let y = MaybeNumber::from(ieee::Single::from_str("0x1p+2").unwrap()).unwrap();
        let z = MaybeNumber::from(ieee::Single::from_str("0x1p+3").unwrap());
        assert_eq!(x.float_mul(y, RoundingFlag::Ceil), z);
    }

    #[test]
    fn float_div() {
        let x = MaybeNumber::from(ieee::Single::from_str("0x1p+0").unwrap()).unwrap();
        let y = MaybeNumber::from(ieee::Single::from_str("0x2p+0").unwrap()).unwrap();
        let z = MaybeNumber::from(ieee::Single::from_str("0x1p-1").unwrap());
        assert_eq!(x.float_div(y, RoundingFlag::Ceil), z);
        let x = MaybeNumber::from(ieee::Single::from_str("0x1p+0").unwrap()).unwrap();
        let y = MaybeNumber::from(ieee::Single::ZERO).unwrap();
        assert_eq!(x.float_div(y, RoundingFlag::Ceil), MaybeNumber::none());
    }

    #[test]
    fn bf16_add() {
        let x = MaybeNumber::from(bf16::from_f32(0.5)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(1.)).unwrap();
        let z = MaybeNumber::from(bf16::from_f32(1.5));
        assert_eq!(x.float_add(y, RoundingFlag::Ceil), z);

        let x = MaybeNumber::from(bf16::from_f32(0.5)).unwrap();
        let y = MaybeNumber::from(bf16::ZERO);
        assert_eq!(x.float_add((-x).unwrap(), RoundingFlag::Ceil), y);

        // will not overflow
        let x = MaybeNumber::from(bf16::MAX).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(1.)).unwrap();
        assert_eq!(x.float_add(y, RoundingFlag::Ceil), MaybeNumber::from(x));
    }

    #[test]
    fn bf16_sub() {
        let x = MaybeNumber::from(bf16::from_f32(0.5)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(1.)).unwrap();
        let z = MaybeNumber::from(bf16::from_f32(-0.5));
        assert_eq!(x.float_sub(y, RoundingFlag::Ceil), z);
    }

    #[test]
    fn bf16_mul() {
        let x = MaybeNumber::from(bf16::from_f32(2.5)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(2.)).unwrap();
        let z = MaybeNumber::from(bf16::from_f32(5.0));
        assert_eq!(x.float_mul(y, RoundingFlag::Ceil), z);
    }

    #[test]
    fn bf16_div() {
        let x = MaybeNumber::from(bf16::from_f32(6.)).unwrap();
        let y = MaybeNumber::from(bf16::from_f32(2.)).unwrap();
        let z = MaybeNumber::from(bf16::from_f32(3.));
        assert_eq!(x.float_div(y, RoundingFlag::Ceil), z);
        let x = MaybeNumber::from(bf16::from_f32(6.)).unwrap();
        let y = MaybeNumber::from(bf16::ZERO).unwrap();
        let z = MaybeNumber::from(bf16::INFINITY);
        assert_eq!(x.float_div(y, RoundingFlag::Ceil), z);
    }
}
