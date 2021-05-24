// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::u512;
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

    pub fn step_op(arithm: Arithmetics, step: i8) -> impl Fn(RegVal) -> RegVal {
        move |src| match arithm {
            Arithmetics::IntChecked { signed: false } => RegVal::step_uint_checked(src, step),
            Arithmetics::IntUnchecked { signed: false } => RegVal::step_uint_unchecked(src, step),
            Arithmetics::IntArbitraryPrecision { signed: false } => RegVal::step_uint_ap(src, step),
            Arithmetics::IntChecked { signed: true } => RegVal::step_int_checked(src, step),
            Arithmetics::IntUnchecked { signed: true } => RegVal::step_int_unchecked(src, step),
            Arithmetics::IntArbitraryPrecision { signed: true } => RegVal::step_int_ap(src, step),
            Arithmetics::Float => RegVal::step_float(src, step),
            Arithmetics::FloatArbitraryPrecision => RegVal::step_float_ap(src, step),
        }
    }

    pub fn add_op(arithm: Arithmetics) -> fn(RegVal, RegVal) -> RegVal {
        match arithm {
            Arithmetics::IntChecked { signed: false } => RegVal::add_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => RegVal::add_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => RegVal::add_uint_ap,
            Arithmetics::IntChecked { signed: true } => RegVal::add_int_checked,
            Arithmetics::IntUnchecked { signed: true } => RegVal::add_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => RegVal::add_int_ap,
            Arithmetics::Float => RegVal::add_float,
            Arithmetics::FloatArbitraryPrecision => RegVal::add_float_ap,
        }
    }

    pub fn sub_op(arithm: Arithmetics) -> fn(RegVal, RegVal) -> RegVal {
        match arithm {
            Arithmetics::IntChecked { signed: false } => RegVal::sub_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => RegVal::sub_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => RegVal::sub_uint_ap,
            Arithmetics::IntChecked { signed: true } => RegVal::sub_int_checked,
            Arithmetics::IntUnchecked { signed: true } => RegVal::mul_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => RegVal::sub_int_ap,
            Arithmetics::Float => RegVal::sub_float,
            Arithmetics::FloatArbitraryPrecision => RegVal::sub_float_ap,
        }
    }

    pub fn mul_op(arithm: Arithmetics) -> fn(RegVal, RegVal) -> RegVal {
        match arithm {
            Arithmetics::IntChecked { signed: false } => RegVal::mul_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => RegVal::mul_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => RegVal::mul_uint_ap,
            Arithmetics::IntChecked { signed: true } => RegVal::mul_int_checked,
            Arithmetics::IntUnchecked { signed: true } => RegVal::mul_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => RegVal::mul_int_ap,
            Arithmetics::Float => RegVal::mul_float,
            Arithmetics::FloatArbitraryPrecision => RegVal::mul_float_ap,
        }
    }

    pub fn div_op(arithm: Arithmetics) -> fn(RegVal, RegVal) -> RegVal {
        match arithm {
            Arithmetics::IntChecked { signed: false } => RegVal::div_uint_checked,
            Arithmetics::IntUnchecked { signed: false } => RegVal::div_uint_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: false } => RegVal::div_uint_ap,
            Arithmetics::IntChecked { signed: true } => RegVal::div_int_checked,
            Arithmetics::IntUnchecked { signed: true } => RegVal::div_int_unchecked,
            Arithmetics::IntArbitraryPrecision { signed: true } => RegVal::div_int_ap,
            Arithmetics::Float => RegVal::div_float,
            Arithmetics::FloatArbitraryPrecision => RegVal::div_float_ap,
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
        self.as_clean().bytes.cmp(&other.as_clean().bytes)
    }

    /// Compares two values according to unsigned arithmetics
    pub fn cmp_int(self, other: Self) -> Ordering {
        let mut a = self.as_clean();
        let mut b = other.as_clean();
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

impl RegVal {
    pub fn step_uint_checked(value: RegVal, step: i8) -> RegVal {
        let value = if let Some(value) = *value {
            value
        } else {
            return RegVal::none();
        };
        let u512_max = u512::from_le_bytes([0xFF; 64]);
        let step = u512::from_u64(step as u64).unwrap();
        let mut val: u512 = value.into();
        if step >= u512_max - val {
            None
        } else {
            val = val + step;
            Some(Value::from(val))
        }
        .into()
    }

    pub fn step_uint_unchecked(value: RegVal, step: i8) -> RegVal {
        let value = if let Some(value) = *value {
            value
        } else {
            return RegVal::none();
        };
        let u512_max = u512::from_le_bytes([0xFF; 64]);
        let step = u512::from_u64(step as u64).unwrap();
        let mut val: u512 = value.into();
        if step >= u512_max - val {
            Some(Value::from(step - (u512_max - val)))
        } else {
            val = val + step;
            Some(Value::from(val))
        }
        .into()
    }

    pub fn step_uint_ap(src: RegVal, step: i8) -> RegVal {
        todo!()
    }

    pub fn step_int_checked(src: RegVal, step: i8) -> RegVal {
        todo!()
    }

    pub fn step_int_unchecked(src: RegVal, step: i8) -> RegVal {
        todo!()
    }

    pub fn step_int_ap(src: RegVal, step: i8) -> RegVal {
        todo!()
    }

    pub fn step_float(src: RegVal, step: i8) -> RegVal {
        todo!()
    }

    pub fn step_float_ap(src: RegVal, step: i8) -> RegVal {
        todo!()
    }
}

impl RegVal {
    pub fn add_uint_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_uint_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_uint_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_int_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_int_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_int_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_float(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn add_float_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }
}

impl RegVal {
    pub fn sub_uint_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_uint_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_uint_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_int_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_int_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_int_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_float(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn sub_float_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }
}

impl RegVal {
    pub fn mul_uint_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_uint_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_uint_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_int_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_int_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_int_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_float(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn mul_float_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }
}

impl RegVal {
    pub fn div_uint_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_uint_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_uint_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_int_checked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_int_unchecked(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_int_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_float(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn div_float_ap(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }
}
