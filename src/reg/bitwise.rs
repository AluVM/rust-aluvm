// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use super::RegVal;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

impl Not for RegVal {
    type Output = RegVal;

    fn not(self) -> Self::Output {
        todo!()
    }
}

impl BitAnd for RegVal {
    type Output = RegVal;

    fn bitand(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitOr for RegVal {
    type Output = RegVal;

    fn bitor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl BitXor for RegVal {
    type Output = RegVal;

    fn bitxor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shl for RegVal {
    type Output = RegVal;

    fn shl(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Shr for RegVal {
    type Output = RegVal;

    fn shr(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl RegVal {
    pub fn scl(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }

    pub fn scr(src1: RegVal, src2: RegVal) -> RegVal {
        todo!()
    }
}
