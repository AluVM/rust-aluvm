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

use amplify_num::u512;
use half::bf16;
use rustc_apfloat::ieee;

use super::{MaybeNumber, Number};
use crate::reg::number::{FloatLayout, IntLayout, Layout};

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
    pub fn step_op(arithm: ArithmFlags, step: i8) -> impl Fn(Number) -> Option<Number> {
        move |src| match arithm {
            ArithmFlags::IntChecked { signed: false } => Number::step_uint_checked(src, step),
            ArithmFlags::IntUnchecked { signed: false } => Number::step_uint_unchecked(src, step),
            ArithmFlags::IntArbitraryPrecision { signed: false } => Number::step_uint_ap(src, step),
            ArithmFlags::IntChecked { signed: true } => Number::step_int_checked(src, step),
            ArithmFlags::IntUnchecked { signed: true } => Number::step_int_unchecked(src, step),
            ArithmFlags::IntArbitraryPrecision { signed: true } => Number::step_int_ap(src, step),
            ArithmFlags::Float => Number::step_float(src, step),
            ArithmFlags::FloatArbitraryPrecision => Number::step_float_ap(src, step),
        }
    }

    pub fn add_op(arithm: ArithmFlags) -> fn(Number, Number) -> Option<Number> {
        match arithm {
            ArithmFlags::IntChecked { signed: false } => Number::add_uint_checked,
            ArithmFlags::IntUnchecked { signed: false } => Number::add_uint_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: false } => Number::add_uint_ap,
            ArithmFlags::IntChecked { signed: true } => Number::add_int_checked,
            ArithmFlags::IntUnchecked { signed: true } => Number::add_int_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: true } => Number::add_int_ap,
            ArithmFlags::Float => Number::add_float,
            ArithmFlags::FloatArbitraryPrecision => Number::add_float_ap,
        }
    }

    pub fn sub_op(arithm: ArithmFlags) -> fn(Number, Number) -> Option<Number> {
        match arithm {
            ArithmFlags::IntChecked { signed: false } => Number::sub_uint_checked,
            ArithmFlags::IntUnchecked { signed: false } => Number::sub_uint_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: false } => Number::sub_uint_ap,
            ArithmFlags::IntChecked { signed: true } => Number::sub_int_checked,
            ArithmFlags::IntUnchecked { signed: true } => Number::mul_int_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: true } => Number::sub_int_ap,
            ArithmFlags::Float => Number::sub_float,
            ArithmFlags::FloatArbitraryPrecision => Number::sub_float_ap,
        }
    }

    pub fn mul_op(arithm: ArithmFlags) -> fn(Number, Number) -> Option<Number> {
        match arithm {
            ArithmFlags::IntChecked { signed: false } => Number::mul_uint_checked,
            ArithmFlags::IntUnchecked { signed: false } => Number::mul_uint_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: false } => Number::mul_uint_ap,
            ArithmFlags::IntChecked { signed: true } => Number::mul_int_checked,
            ArithmFlags::IntUnchecked { signed: true } => Number::mul_int_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: true } => Number::mul_int_ap,
            ArithmFlags::Float => Number::mul_float,
            ArithmFlags::FloatArbitraryPrecision => Number::mul_float_ap,
        }
    }

    pub fn div_op(arithm: ArithmFlags) -> fn(Number, Number) -> Option<Number> {
        match arithm {
            ArithmFlags::IntChecked { signed: false } => Number::div_uint_checked,
            ArithmFlags::IntUnchecked { signed: false } => Number::div_uint_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: false } => Number::div_uint_ap,
            ArithmFlags::IntChecked { signed: true } => Number::div_int_checked,
            ArithmFlags::IntUnchecked { signed: true } => Number::div_int_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: true } => Number::div_int_ap,
            ArithmFlags::Float => Number::div_float,
            ArithmFlags::FloatArbitraryPrecision => Number::div_float_ap,
        }
    }

    pub fn rem_op(arithm: ArithmFlags) -> fn(Number, Number) -> Option<Number> {
        match arithm {
            ArithmFlags::IntChecked { signed: false } => Number::rem_uint_checked,
            ArithmFlags::IntUnchecked { signed: false } => Number::rem_uint_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: false } => Number::rem_uint_ap,
            ArithmFlags::IntChecked { signed: true } => Number::rem_int_checked,
            ArithmFlags::IntUnchecked { signed: true } => Number::rem_int_unchecked,
            ArithmFlags::IntArbitraryPrecision { signed: true } => Number::rem_int_ap,
            ArithmFlags::Float => Number::rem_float,
            ArithmFlags::FloatArbitraryPrecision => Number::rem_float_ap,
        }
    }
}
impl Number {
    pub fn step_uint_checked(value: Number, step: i8) -> Option<Number> {
        let u512_max = u512::from_le_bytes([0xFF; 64]);
        let step = u512::from(step as u64);
        let mut val: u512 = value.into();
        if step >= u512_max - val {
            None
        } else {
            val += step;
            Some(Number::from(val))
        }
    }

    pub fn step_uint_unchecked(value: Number, step: i8) -> Option<Number> {
        let u512_max = u512::from_le_bytes([0xFF; 64]);
        let step = u512::from(step as u64);
        let mut val: u512 = value.into();
        if step >= u512_max - val {
            Some(Number::from(step - (u512_max - val)))
        } else {
            val += step;
            Some(Number::from(val))
        }
    }

    pub fn step_uint_ap(src: Number, step: i8) -> Option<Number> { todo!() }

    pub fn step_int_checked(src: Number, step: i8) -> Option<Number> { todo!() }

    pub fn step_int_unchecked(src: Number, step: i8) -> Option<Number> { todo!() }

    pub fn step_int_ap(src: Number, step: i8) -> Option<Number> { todo!() }

    pub fn step_float(src: Number, step: i8) -> Option<Number> { todo!() }

    pub fn step_float_ap(src: Number, step: i8) -> Option<Number> { todo!() }
}

impl Number {
    pub fn add_uint_checked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        src1.checked_add(src2).map(Number::from)
    }

    pub fn add_uint_unchecked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        Some(src1.wrapping_add(src2).into())
    }

    pub fn add_uint_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn add_int_checked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn add_int_unchecked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn add_int_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn add_float(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn add_float_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }
}

impl Number {
    pub fn sub_uint_checked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        src1.checked_sub(src2).map(Number::from)
    }

    pub fn sub_uint_unchecked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        Some(src1.wrapping_sub(src2).into())
    }

    pub fn sub_uint_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn sub_int_checked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn sub_int_unchecked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn sub_int_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn sub_float(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn sub_float_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }
}

impl Number {
    pub fn mul_uint_checked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        src1.checked_mul(src2).map(Number::from)
    }

    pub fn mul_uint_unchecked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        Some(src1.wrapping_mul(src2).into())
    }

    pub fn mul_uint_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn mul_int_checked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn mul_int_unchecked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn mul_int_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn mul_float(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn mul_float_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }
}

impl Number {
    pub fn div_uint_checked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            None
        } else {
            Some((src1 / src2).into())
        }
    }

    pub fn div_uint_unchecked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            Some(0.into())
        } else {
            Some((src1 / src2).into())
        }
    }

    pub fn div_uint_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn div_int_checked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn div_int_unchecked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn div_int_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn div_float(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn div_float_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }
}

impl Number {
    pub fn rem_uint_checked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            None
        } else {
            Some((src1 % src2).into())
        }
    }

    pub fn rem_uint_unchecked(src1: Number, src2: Number) -> Option<Number> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            Some(0.into())
        } else {
            Some((src1 % src2).into())
        }
    }

    pub fn rem_uint_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn rem_int_checked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn rem_int_unchecked(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn rem_int_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn rem_float(src1: Number, src2: Number) -> Option<Number> { todo!() }

    pub fn rem_float_ap(src1: Number, src2: Number) -> Option<Number> { todo!() }
}
