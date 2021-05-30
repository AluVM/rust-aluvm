// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::convert::TryFrom;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

use super::{MaybeNumber, Number};

impl Not for MaybeNumber {
    type Output = MaybeNumber;

    #[inline]
    fn not(self) -> Self::Output { self.map(Number::not).into() }
}

impl Not for Number {
    type Output = Number;

    #[inline]
    fn not(mut self) -> Self::Output {
        for i in 0..self.len() {
            self[i] = !self[i];
        }
        self
    }
}

impl BitAnd for Number {
    type Output = Number;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.to_u1024_bytes().bitand(rhs.to_u1024_bytes()).into()
    }
}

impl BitOr for Number {
    type Output = Number;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.to_u1024_bytes().bitor(rhs.to_u1024_bytes()).into()
    }
}

impl BitXor for Number {
    type Output = Number;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.to_u1024_bytes().bitxor(rhs.to_u1024_bytes()).into()
    }
}

impl Shl for Number {
    type Output = Number;

    #[inline]
    fn shl(self, rhs: Self) -> Self::Output {
        self.to_u1024_bytes()
            .shl(u16::try_from(rhs).expect("attempt to bitshift left for more than 2^16 bits")
                as usize)
            .into()
    }
}

impl Shr for Number {
    type Output = Number;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        self.to_u1024_bytes()
            .shr(u16::try_from(rhs).expect("attempt to bitshift right for more than 2^16 bits")
                as usize)
            .into()
    }
}

impl Number {
    pub fn scl(src1: Number, src2: Number) -> Number { todo!() }

    pub fn scr(src1: Number, src2: Number) -> Number { todo!() }
}
