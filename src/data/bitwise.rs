// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2023 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023 UBIDECO Institute. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::convert::TryFrom;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

use amplify::num::{i1024, u1024};

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
        let layout = self.layout();
        assert!(layout.is_integer(), "bit shifting float number");
        let rhs = u16::try_from(rhs).expect("attempt to bitshift lhs for more than 2^16 bits");
        let mut n = match layout.is_signed_int() {
            true => {
                Number::from(self.to_i1024_bytes().checked_shl(rhs as u32).unwrap_or(i1024::ZERO))
            }
            false => {
                Number::from(self.to_u1024_bytes().checked_shl(rhs as u32).unwrap_or(u1024::ZERO))
            }
        };
        n.reshape(layout);
        n
    }
}

impl Shr for Number {
    type Output = Number;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        let layout = self.layout();
        assert!(layout.is_integer(), "bit shifting float number");
        let rhs = u16::try_from(rhs).expect("attempt to bitshift right for more than 2^16 bits");
        let mut n = match layout.is_signed_int() {
            true => {
                Number::from(self.to_i1024_bytes().checked_shr(rhs as u32).unwrap_or(i1024::ZERO))
            }
            false => {
                Number::from(self.to_u1024_bytes().checked_shr(rhs as u32).unwrap_or(u1024::ZERO))
            }
        };
        n.reshape(layout);
        n
    }
}

impl Number {
    /// Cyclic bit shift left. Panics if the number is not an integer.
    pub fn scl(self, shift: Number) -> Number {
        let layout = self.layout();
        let bits = self.len() * 8;
        let lhs = self.into_unsigned();
        assert!(layout.is_integer(), "bit shifting float number");
        let excess = u16::try_from(shift).map(|v| v % bits).expect(
            "shift value in `scl` operation must always be from either `a8` or `a16` registry",
        );
        let residue = lhs >> Number::from(bits - excess);
        ((lhs << Number::from(excess)) | residue).reshaped(layout, true).expect("restoring layout")
    }

    /// Cyclic bit shift right. Panics if the number is not an integer.
    pub fn scr(self, shift: Number) -> Number {
        let layout = self.layout();
        let bits = self.len() * 8;
        let lhs = self.into_unsigned();
        assert!(layout.is_integer(), "bit shifting float number");
        let excess = u16::try_from(shift).map(|v| v % bits).expect(
            "shift value in `scl` operation must always be from either `a8` or `a16` registry",
        );
        let residue = lhs << Number::from(bits - excess);
        ((lhs >> Number::from(excess)) | residue).reshaped(layout, true).expect("restoring layout")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shl_test() {
        let x = Number::from(6u8);
        let y = Number::from(24u8);
        assert_eq!(x.shl(Number::from(2)), y);
        let x = Number::from(-1i16);
        let y = Number::from(-2i16);
        assert_eq!(x.shl(Number::from(1)), y);
    }

    #[test]
    fn shr_test() {
        let x = Number::from(9u8);
        let y = Number::from(4u8);
        assert_eq!(x.shr(Number::from(1)), y);
        let x = Number::from(-2i16);
        let y = Number::from(-1i16);
        assert_eq!(x.shr(Number::from(1)), y);
    }

    #[test]
    fn scl_test() {
        let x = Number::from(131u8);
        let y = Number::from(7u8);
        assert_eq!(x.scl(Number::from(1)), y);
        let x = Number::from(-7i16);
        let y = Number::from(-25i16);
        assert_eq!(x.scl(Number::from(2)), y);
    }

    #[test]
    fn scr_test() {
        let x = Number::from(129u8);
        let y = Number::from(192u8);
        assert_eq!(x.scr(Number::from(1)), y);
        let x = Number::from(1i8);
        let y = Number::from(64i8);
        assert_eq!(x.scr(Number::from(2)), y);
    }

    #[test]
    fn reverse_bits_test() {
        let x = Number::from(192u8);
        let y = Number::from(3u8);
        assert_eq!(x.reverse_bits(), y);
        let x = Number::from(1i8);
        let y = Number::from(-128i8);
        assert_eq!(x.reverse_bits(), y);
    }
}
