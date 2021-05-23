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

use super::{RegVal, Value};
use crate::instr::Arithmetics;

impl RegVal {
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
