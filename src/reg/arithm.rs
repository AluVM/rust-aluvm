// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify_num::u512;
use core::cmp::Ordering;

use super::{RegVal, Value};
use crate::instr::{Arithmetics, NumType};

impl RegVal {
    pub fn partial_cmp_op(num_type: NumType) -> fn(RegVal, RegVal) -> Option<Ordering> {
        match num_type {
            NumType::Unsigned => RegVal::partial_cmp_uint,
            NumType::Signed => RegVal::partial_cmp_int,
            NumType::Float23 => RegVal::partial_cmp_f23,
            NumType::Float52 => RegVal::partial_cmp_f52,
        }
    }
}

impl Value {
    pub fn step_op(arithm: Arithmetics, step: i8) -> impl Fn(Value) -> Option<Value> {
        move |src| match arithm {
            Arithmetics::IntChecked { signed: false } => Value::step_uint_checked(src, step),
            Arithmetics::IntUnchecked { signed: false } => Value::step_uint_unchecked(src, step),
            Arithmetics::IntArbitraryPrecision { signed: false } => Value::step_uint_ap(src, step),
            Arithmetics::IntChecked { signed: true } => Value::step_int_checked(src, step),
            Arithmetics::IntUnchecked { signed: true } => Value::step_int_unchecked(src, step),
            Arithmetics::IntArbitraryPrecision { signed: true } => Value::step_int_ap(src, step),
            Arithmetics::Float => Value::step_float(src, step),
            Arithmetics::FloatArbitraryPrecision => Value::step_float_ap(src, step),
        }
    }

    pub fn add_op(arithm: Arithmetics) -> fn(Value, Value) -> Option<Value> {
        match arithm {
            Arithmetics::IntChecked { signed: false } => Value::add_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => Value::add_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => Value::add_uint_ap,
            Arithmetics::IntChecked { signed: true } => Value::add_int_checked,
            Arithmetics::IntUnchecked { signed: true } => Value::add_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => Value::add_int_ap,
            Arithmetics::Float => Value::add_float,
            Arithmetics::FloatArbitraryPrecision => Value::add_float_ap,
        }
    }

    pub fn sub_op(arithm: Arithmetics) -> fn(Value, Value) -> Option<Value> {
        match arithm {
            Arithmetics::IntChecked { signed: false } => Value::sub_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => Value::sub_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => Value::sub_uint_ap,
            Arithmetics::IntChecked { signed: true } => Value::sub_int_checked,
            Arithmetics::IntUnchecked { signed: true } => Value::mul_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => Value::sub_int_ap,
            Arithmetics::Float => Value::sub_float,
            Arithmetics::FloatArbitraryPrecision => Value::sub_float_ap,
        }
    }

    pub fn mul_op(arithm: Arithmetics) -> fn(Value, Value) -> Option<Value> {
        match arithm {
            Arithmetics::IntChecked { signed: false } => Value::mul_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => Value::mul_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => Value::mul_uint_ap,
            Arithmetics::IntChecked { signed: true } => Value::mul_int_checked,
            Arithmetics::IntUnchecked { signed: true } => Value::mul_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => Value::mul_int_ap,
            Arithmetics::Float => Value::mul_float,
            Arithmetics::FloatArbitraryPrecision => Value::mul_float_ap,
        }
    }

    pub fn div_op(arithm: Arithmetics) -> fn(Value, Value) -> Option<Value> {
        match arithm {
            Arithmetics::IntChecked { signed: false } => Value::div_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => Value::div_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => Value::div_uint_ap,
            Arithmetics::IntChecked { signed: true } => Value::div_int_checked,
            Arithmetics::IntUnchecked { signed: true } => Value::div_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => Value::div_int_ap,
            Arithmetics::Float => Value::div_float,
            Arithmetics::FloatArbitraryPrecision => Value::div_float_ap,
        }
    }

    pub fn rem_op(arithm: Arithmetics) -> fn(Value, Value) -> Option<Value> {
        match arithm {
            Arithmetics::IntChecked { signed: false } => Value::rem_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => Value::rem_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => Value::rem_uint_ap,
            Arithmetics::IntChecked { signed: true } => Value::rem_int_checked,
            Arithmetics::IntUnchecked { signed: true } => Value::rem_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => Value::rem_int_ap,
            Arithmetics::Float => Value::rem_float,
            Arithmetics::FloatArbitraryPrecision => Value::rem_float_ap,
        }
    }
}

impl RegVal {
    /// Compares two values according to given arithmetics
    pub fn partial_cmp(self, num_type: NumType, other: Self) -> Option<Ordering> {
        match num_type {
            NumType::Unsigned => self.partial_cmp_uint(other),
            NumType::Signed => self.partial_cmp_int(other),
            NumType::Float23 => self.partial_cmp_f23(other),
            NumType::Float52 => self.partial_cmp_f52(other),
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

impl Value {
    /// Compares two values according to given arithmetics
    pub fn cmp(self, num_type: NumType, other: Self) -> Ordering {
        match num_type {
            NumType::Unsigned => self.cmp_uint(other),
            NumType::Signed => self.cmp_int(other),
            NumType::Float23 => self.cmp_f23(other),
            NumType::Float52 => self.cmp_f52(other),
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
    pub fn cmp_f23(self, other: Self) -> Ordering {
        todo!("short float comparison")
    }

    /// Compares two values according to long float arithmetics
    pub fn cmp_f52(self, other: Self) -> Ordering {
        todo!("short long comparison")
    }
}

impl Value {
    pub fn step_uint_checked(value: Value, step: i8) -> Option<Value> {
        let u512_max = u512::from_le_bytes([0xFF; 64]);
        let step = u512::from(step as u64);
        let mut val: u512 = value.into();
        if step >= u512_max - val {
            None
        } else {
            val = val + step;
            Some(Value::from(val))
        }
    }

    pub fn step_uint_unchecked(value: Value, step: i8) -> Option<Value> {
        let u512_max = u512::from_le_bytes([0xFF; 64]);
        let step = u512::from(step as u64);
        let mut val: u512 = value.into();
        if step >= u512_max - val {
            Some(Value::from(step - (u512_max - val)))
        } else {
            val = val + step;
            Some(Value::from(val))
        }
    }

    pub fn step_uint_ap(src: Value, step: i8) -> Option<Value> {
        todo!()
    }

    pub fn step_int_checked(src: Value, step: i8) -> Option<Value> {
        todo!()
    }

    pub fn step_int_unchecked(src: Value, step: i8) -> Option<Value> {
        todo!()
    }

    pub fn step_int_ap(src: Value, step: i8) -> Option<Value> {
        todo!()
    }

    pub fn step_float(src: Value, step: i8) -> Option<Value> {
        todo!()
    }

    pub fn step_float_ap(src: Value, step: i8) -> Option<Value> {
        todo!()
    }
}

impl Value {
    pub fn add_uint_checked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        src1.checked_add(src2).map(Value::from)
    }

    pub fn add_uint_unchecked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        Some(src1.wrapping_add(src2).into())
    }

    pub fn add_uint_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn add_int_checked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn add_int_unchecked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn add_int_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn add_float(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn add_float_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }
}

impl Value {
    pub fn sub_uint_checked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        src1.checked_sub(src2).map(Value::from)
    }

    pub fn sub_uint_unchecked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        Some(src1.wrapping_sub(src2).into())
    }

    pub fn sub_uint_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn sub_int_checked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn sub_int_unchecked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn sub_int_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn sub_float(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn sub_float_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }
}

impl Value {
    pub fn mul_uint_checked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        src1.checked_mul(src2).map(Value::from)
    }

    pub fn mul_uint_unchecked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        Some(src1.wrapping_mul(src2).into())
    }

    pub fn mul_uint_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn mul_int_checked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn mul_int_unchecked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn mul_int_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn mul_float(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn mul_float_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }
}

impl Value {
    pub fn div_uint_checked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            None
        } else {
            Some((src1 / src2).into())
        }
    }

    pub fn div_uint_unchecked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            Some(0.into())
        } else {
            Some((src1 / src2).into())
        }
    }

    pub fn div_uint_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn div_int_checked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn div_int_unchecked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn div_int_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn div_float(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn div_float_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }
}

impl Value {
    pub fn rem_uint_checked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            None
        } else {
            Some((src1 % src2).into())
        }
    }

    pub fn rem_uint_unchecked(src1: Value, src2: Value) -> Option<Value> {
        let src1: u512 = src1.into();
        let src2: u512 = src2.into();
        if src2 == u512::ZERO {
            Some(0.into())
        } else {
            Some((src1 % src2).into())
        }
    }

    pub fn rem_uint_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn rem_int_checked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn rem_int_unchecked(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn rem_int_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn rem_float(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }

    pub fn rem_float_ap(src1: Value, src2: Value) -> Option<Value> {
        todo!()
    }
}
