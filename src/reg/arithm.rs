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

use super::{MaybeNumber, Number};

impl MaybeNumber {
    pub fn partial_cmp_op(num_type: CmpFlag) -> fn(MaybeNumber, MaybeNumber) -> Option<Ordering> {
        match num_type {
            CmpFlag::Unsigned => MaybeNumber::partial_cmp_uint,
            CmpFlag::Signed => MaybeNumber::partial_cmp_int,
            CmpFlag::ExactEq => MaybeNumber::partial_cmp_f23,
            CmpFlag::RoundingEq => MaybeNumber::partial_cmp_f52,
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

impl MaybeNumber {
    /// Compares two values according to given arithmetics
    pub fn partial_cmp(self, num_type: CmpFlag, other: Self) -> Option<Ordering> {
        match num_type {
            CmpFlag::Unsigned => self.partial_cmp_uint(other),
            CmpFlag::Signed => self.partial_cmp_int(other),
            CmpFlag::ExactEq => self.partial_cmp_f23(other),
            CmpFlag::RoundingEq => self.partial_cmp_f52(other),
        }
    }

    /// Compares two values according to unsigned arithmetics
    pub fn partial_cmp_uint(self, other: Self) -> Option<Ordering> {
        match (*self, *other) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) | (Some(_), None) => None,
            (Some(a), Some(b)) => Some(a.cmp_uint(b)),
        }
    }

    /// Compares two values according to unsigned arithmetics
    pub fn partial_cmp_int(self, other: Self) -> Option<Ordering> {
        match (*self, *other) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) | (Some(_), None) => None,
            (Some(a), Some(b)) => Some(a.cmp_int(b)),
        }
    }

    /// Compares two values according to short float arithmetics
    pub fn partial_cmp_f23(self, other: Self) -> Option<Ordering> {
        match (*self, *other) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) | (Some(_), None) => None,
            (Some(a), Some(b)) => Some(a.cmp_f23(b)),
        }
    }

    /// Compares two values according to long float arithmetics
    pub fn partial_cmp_f52(self, other: Self) -> Option<Ordering> {
        match (*self, *other) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) | (Some(_), None) => None,
            (Some(a), Some(b)) => Some(a.cmp_f52(b)),
        }
    }
}

impl Number {
    /// Compares two values according to given arithmetics
    pub fn cmp(self, num_type: CmpFlag, other: Self) -> Ordering {
        match num_type {
            CmpFlag::Unsigned => self.cmp_uint(other),
            CmpFlag::Signed => self.cmp_int(other),
            CmpFlag::ExactEq => self.cmp_f23(other),
            CmpFlag::RoundingEq => self.cmp_f52(other),
        }
    }

    /// Compares two values according to unsigned arithmetics
    pub fn cmp_uint(self, other: Self) -> Ordering {
        self.to_clean().bytes.cmp(&other.to_clean().bytes)
    }

    /// Compares two values according to unsigned arithmetics
    pub fn cmp_int(self, other: Self) -> Ordering {
        let mut a = self.to_clean();
        let mut b = other.to_clean();
        let rev_a = if a.len > 0 {
            let sign = &mut a.bytes[a.len as usize - 1];
            let rev_a = *sign & 0x80 == 0x80;
            *sign &= 0x7F;
            rev_a
        } else {
            false
        };
        let rev_b = if b.len > 0 {
            let sign = &mut b.bytes[b.len as usize - 1];
            let rev_b = *sign & 0x80 == 0x80;
            *sign &= 0x7F;
            rev_b
        } else {
            false
        };
        match (rev_a, rev_b) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => a.bytes.cmp(&b.bytes),
            (true, true) => a.bytes.cmp(&b.bytes).reverse(),
        }
    }

    /// Compares two values according to short float arithmetics
    pub fn cmp_f23(self, other: Self) -> Ordering { todo!("short float comparison") }

    /// Compares two values according to long float arithmetics
    pub fn cmp_f52(self, other: Self) -> Ordering { todo!("short long comparison") }
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
