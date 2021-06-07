// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::cmp::Ordering;
use core::convert::TryFrom;
use core::ops::{Div, Neg, Rem};

use half::bf16;
use rustc_apfloat::{ieee, Float, Status};

use super::Number;
use crate::instr::{IntFlags, RoundingFlag};
use crate::reg::number::{FloatLayout, IntLayout, Layout, NumberLayout};

impl PartialEq for Number {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.layout() == other.layout()
            || self.layout().is_signed_int() && other.layout().is_unsigned_int()
            || self.layout().is_unsigned_int() && other.layout().is_signed_int())
            && self.to_clean().eq(&other.to_clean())
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
    ///
    /// Panics if applied to float number layouts.
    pub fn int_add(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "adding numbers with different layout");
        match layout {
            // Signed and unsigned integers do not differ in their addition, since we use
            // two's complement system
            Layout::Integer(IntLayout { .. }) => {
                let val1 = self.to_u1024_bytes();
                let val2 = rhs.to_u1024_bytes();
                let (res, overflow) = val1.overflowing_add(val2);
                if !overflow || flags.wrap {
                    Some(Number::from(res))
                } else {
                    None
                }
            }
            Layout::Float(_) => panic!("integer addition of float numbers"),
        }
    }

    /// Subtraction of two integers with configuration flags for overflow and signed format.
    ///
    /// Panics if applied to float number layouts.
    pub fn int_sub(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "subtracting numbers with different layout");
        match layout {
            // Signed and unsigned integers do not differ in their subtraction, since we use
            // two's complement system
            Layout::Integer(IntLayout { .. }) => {
                let val1 = self.to_u1024_bytes();
                let val2 = rhs.to_u1024_bytes();
                let (res, overflow) = val1.overflowing_sub(val2);
                if !overflow || flags.wrap {
                    Some(Number::from(res))
                } else {
                    None
                }
            }
            Layout::Float(_) => panic!("integer subtraction of float numbers"),
        }
    }

    /// Multiplication of two integers with configuration flags for overflow and signed format.
    ///
    /// Panics if applied to float number layouts.
    pub fn int_mul(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "multiplying numbers with different layout");
        match layout {
            Layout::Integer(IntLayout { signed: true, .. }) => {
                let val1 = self.to_u1024_bytes();
                let val2 = rhs.to_u1024_bytes();
                let (res, overflow) = val1.overflowing_mul(val2);
                if !overflow || flags.wrap {
                    Some(Number::from(res))
                } else {
                    None
                }
            }
            Layout::Integer(IntLayout { signed: false, .. }) if layout.bits() <= 128 => {
                let val1 = i128::try_from(self).expect("integer layout is broken");
                let val2 = i128::try_from(rhs).expect("integer layout is broken");
                let (res, overflow) = val1.overflowing_mul(val2);
                if !overflow || flags.wrap {
                    Some(Number::from(res))
                } else {
                    None
                }
            }
            Layout::Integer(IntLayout { signed: false, .. }) => {
                todo!("implement booth multiplication algorithm")
            }
            Layout::Float(_) => panic!("integer multiplication of float numbers"),
        }
    }

    /// Division of two integers with configuration flags for overflow and signed format.
    ///
    /// Panics if applied to float number layouts.
    pub fn int_div(self, rhs: Self, flags: IntFlags) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "dividing numbers with different layout");

        if self.is_zero() && rhs.is_zero() {
            return None;
        }
        if rhs.is_zero() {
            if flags.wrap {
                return Some(Number::zero(layout));
            } else {
                return None;
            }
        }

        if flags.wrap {
            todo!("euclidean division in amplify crate")
        }

        Some(match layout {
            Layout::Integer(IntLayout { signed: true, .. }) => {
                let val1 = self.to_u1024_bytes();
                let val2 = rhs.to_u1024_bytes();
                val1.div(val2).into()
            }
            Layout::Integer(IntLayout { signed: false, .. }) if layout.bits() <= 128 => {
                let val1 = i128::try_from(self).expect("integer layout is broken");
                let val2 = i128::try_from(rhs).expect("integer layout is broken");
                val1.div(val2).into()
            }
            Layout::Integer(IntLayout { signed: false, .. }) => {
                todo!("implement large signed number division algorithm")
            }
            Layout::Float(_) => panic!("integer division of float numbers"),
        })
    }

    /// Addition of two floats with configuration flags for rounding.
    ///
    /// Panics if applied to integer number layouts.
    pub fn float_add(self, rhs: Self, flag: RoundingFlag) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "adding numbers with different layout");
        let (val, status) = match layout {
            Layout::Float(FloatLayout::BFloat16) => todo!("addition of BF16 floats"),
            Layout::Float(FloatLayout::IeeeHalf) => {
                let val1 = ieee::Half::from(self);
                let val2 = ieee::Half::from(rhs);
                let res = val1.add_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                let val1 = ieee::Single::from(self);
                let val2 = ieee::Single::from(rhs);
                let res = val1.add_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                let val1 = ieee::Double::from(self);
                let val2 = ieee::Double::from(rhs);
                let res = val1.add_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                let val1 = ieee::Quad::from(self);
                let val2 = ieee::Quad::from(rhs);
                let res = val1.add_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                let val1 = ieee::X87DoubleExtended::from(self);
                let val2 = ieee::X87DoubleExtended::from(rhs);
                let res = val1.add_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeOct) => unimplemented!("256-bit floats"),
            Layout::Float(FloatLayout::FloatTapered) => todo!("addition of tapered floats"),
            Layout::Integer(_) => panic!("float addition of integer numbers"),
        };
        match (val.is_nan(), status) {
            (true, _) => None,
            (false, Status::OVERFLOW | Status::INEXACT) => None,
            _ => Some(val),
        }
    }

    /// Subtraction of two floats with configuration flags for rounding.
    ///
    /// Panics if applied to integer number layouts.
    pub fn float_sub(self, rhs: Self, flag: RoundingFlag) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "subtracting numbers with different layout");
        let (val, status) = match layout {
            Layout::Float(FloatLayout::BFloat16) => todo!("subtraction of BF16 floats"),
            Layout::Float(FloatLayout::IeeeHalf) => {
                let val1 = ieee::Half::from(self);
                let val2 = ieee::Half::from(rhs);
                let res = val1.sub_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                let val1 = ieee::Single::from(self);
                let val2 = ieee::Single::from(rhs);
                let res = val1.sub_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                let val1 = ieee::Double::from(self);
                let val2 = ieee::Double::from(rhs);
                let res = val1.sub_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                let val1 = ieee::Quad::from(self);
                let val2 = ieee::Quad::from(rhs);
                let res = val1.sub_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                let val1 = ieee::X87DoubleExtended::from(self);
                let val2 = ieee::X87DoubleExtended::from(rhs);
                let res = val1.sub_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeOct) => unimplemented!("256-bit floats"),
            Layout::Float(FloatLayout::FloatTapered) => todo!("subtraction of tapered floats"),
            Layout::Integer(_) => panic!("float subtraction of integer numbers"),
        };
        match (val.is_nan(), status) {
            (true, _) => None,
            (false, Status::OVERFLOW | Status::INEXACT) => None,
            _ => Some(val),
        }
    }

    /// Multiplication of two floats with configuration flags for rounding.
    ///
    /// Panics if applied to integer number layouts.
    pub fn float_mul(self, rhs: Self, flag: RoundingFlag) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "multiplying numbers with different layout");
        let (val, status) = match layout {
            Layout::Float(FloatLayout::BFloat16) => todo!("multiplication of BF16 floats"),
            Layout::Float(FloatLayout::IeeeHalf) => {
                let val1 = ieee::Half::from(self);
                let val2 = ieee::Half::from(rhs);
                let res = val1.mul_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                let val1 = ieee::Single::from(self);
                let val2 = ieee::Single::from(rhs);
                let res = val1.mul_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                let val1 = ieee::Double::from(self);
                let val2 = ieee::Double::from(rhs);
                let res = val1.mul_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                let val1 = ieee::Quad::from(self);
                let val2 = ieee::Quad::from(rhs);
                let res = val1.mul_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                let val1 = ieee::X87DoubleExtended::from(self);
                let val2 = ieee::X87DoubleExtended::from(rhs);
                let res = val1.mul_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeOct) => unimplemented!("256-bit floats"),
            Layout::Float(FloatLayout::FloatTapered) => todo!("multiplication of tapered floats"),
            Layout::Integer(_) => panic!("float multiplication of integer numbers"),
        };
        match (val.is_nan(), status) {
            (true, _) => None,
            (false, Status::OVERFLOW | Status::INEXACT) => None,
            _ => Some(val),
        }
    }

    /// Division of two floats with configuration flags for rounding.
    ///
    /// Panics if applied to integer number layouts.
    pub fn float_div(self, rhs: Self, flag: RoundingFlag) -> Option<Number> {
        let layout = self.layout();
        assert_eq!(layout, rhs.layout(), "dividing numbers with different layout");
        let (val, status) = match layout {
            Layout::Float(FloatLayout::BFloat16) => todo!("division of BF16 floats"),
            Layout::Float(FloatLayout::IeeeHalf) => {
                let val1 = ieee::Half::from(self);
                let val2 = ieee::Half::from(rhs);
                let res = val1.div_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeSingle) => {
                let val1 = ieee::Single::from(self);
                let val2 = ieee::Single::from(rhs);
                let res = val1.div_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeDouble) => {
                let val1 = ieee::Double::from(self);
                let val2 = ieee::Double::from(rhs);
                let res = val1.div_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeQuad) => {
                let val1 = ieee::Quad::from(self);
                let val2 = ieee::Quad::from(rhs);
                let res = val1.div_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::X87DoubleExt) => {
                let val1 = ieee::X87DoubleExtended::from(self);
                let val2 = ieee::X87DoubleExtended::from(rhs);
                let res = val1.div_r(val2, flag.into());
                (Number::from(res.value), res.status)
            }
            Layout::Float(FloatLayout::IeeeOct) => unimplemented!("256-bit floats"),
            Layout::Float(FloatLayout::FloatTapered) => todo!("division of tapered floats"),
            Layout::Integer(_) => panic!("float division of integer numbers"),
        };
        match (val.is_nan(), status) {
            (true, _) => None,
            (false, Status::DIV_BY_ZERO) => None,
            (false, Status::INVALID_OP) => None,
            (false, Status::OVERFLOW | Status::INEXACT) => None,
            _ => Some(val),
        }
    }
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
                let val1 = i128::try_from(self).expect("integer layout is broken");
                let val2 = i128::try_from(rhs).expect("integer layout is broken");
                val1.rem(val2).into()
            }
            Layout::Integer(IntLayout { .. }) => {
                todo!("implement large signed number modulo division algorithm")
            }
            Layout::Float(_) => panic!("modulo division of float number"),
        })
    }
}

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Self::Output { self.applying_sign(self.is_positive()) }
}

impl Number {
    /// Returns the absolute value of the number
    pub fn abs(self) -> Number {
        if self.is_positive() {
            self
        } else {
            self.applying_sign(false)
        }
    }
}
