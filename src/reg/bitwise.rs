// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use super::{RegVal, Value};
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

impl Not for RegVal {
    type Output = RegVal;

    fn not(self) -> Self::Output {
        todo!()
    }
}

impl BitAnd for Value {
    type Output = Value;

    fn bitand(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitOr for Value {
    type Output = Value;

    fn bitor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitXor for Value {
    type Output = Value;

    fn bitxor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shl for Value {
    type Output = Value;

    fn shl(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shr for Value {
    type Output = Value;

    fn shr(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Value {
    pub fn scl(src1: Value, src2: Value) -> Value {
        todo!()
    }

    pub fn scr(src1: Value, src2: Value) -> Value {
        todo!()
    }
}
