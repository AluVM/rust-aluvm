// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://github.com/internet2-org/aluvm-spec>
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::convert::TryFrom;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

use super::{MaybeNumber, Number};
use crate::reg::number::NumberLayout;

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
        assert!(self.layout().is_integer(), "bit shifting float number");
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
        assert!(self.layout().is_integer(), "bit shifting float number");
        self.to_u1024_bytes()
            .shr(u16::try_from(rhs).expect("attempt to bitshift right for more than 2^16 bits")
                as usize)
            .into()
    }
}

impl Number {
    /// Cyclic bit shift left. Panics if the number is not an integer.
    pub fn scl(self, shift: Number) -> Number {
        assert!(self.layout().is_integer(), "bit shifting float number");
        let excess = u16::try_from(shift).expect(
            "shift value in `scl` operation must always be from either `a8` or `a16` registry",
        );
        let residue = self >> Number::from(self.len() - excess);
        (self << shift) | residue
    }

    /// Cyclic bit shift right. Panics if the number is not an integer.
    pub fn scr(self, shift: Number) -> Number {
        assert!(self.layout().is_integer(), "bit shifting float number");
        let excess = u16::try_from(shift).expect(
            "shift value in `scl` operation must always be from either `a8` or `a16` registry",
        );
        let residue = self << Number::from(self.len() - excess);
        (self >> shift) | residue
    }

    /// Bit shift right for signed value. Panics if the number is not an integer.
    pub fn shr_signed(self, shift: Number) -> Number {
        assert!(self.layout().is_integer(), "bit shifting float number");
        if self.layout().bits() > 128 {
            todo!("implement signed right bit shift")
        }
        let shift = u16::try_from(shift).expect(
            "shift value in `scl` operation must always be from either `a8` or `a16` registry",
        );
        let val = i128::from(self);
        Number::from(val << shift)
    }

    /// Reverses the order of bits in the integer. The least significant bit becomes the most
    /// significant bit, second least-significant bit becomes second most-significant bit, etc.
    pub fn reverse_bits(mut self) -> Number {
        assert!(self.layout().is_integer(), "reversing bit order of float");
        let bytes = &mut self[..];
        bytes.reverse();
        bytes.iter_mut().for_each(|byte| *byte = byte.reverse_bits());
        self
    }
}
