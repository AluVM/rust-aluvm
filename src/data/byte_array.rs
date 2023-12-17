// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023 by
//     Yudai Kiyofuji <own7000hr@gmail.com>
//
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

//! Module defining ByteArray layout (integer, signed/unsigned, float etc) and universal in-memory
//! ByteArray representation.

use alloc::string::{String, ToString};
use core::fmt::{self, Debug, Formatter, Write};
use core::hash::{Hash, Hasher};
use core::ops::{
    Deref, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use amplify::num::apfloat::{Status, StatusAnd};

use crate::data::{Layout, Number, NumberLayout};

/// Representation of the value from a register, which may be `None` if the register is unset.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, From)]
pub struct MaybeByteArray(Option<ByteArray>);

impl MaybeByteArray {
    /// Creates [`MaybeByteArray`] without assigning a value to it
    #[inline]
    pub fn none() -> MaybeByteArray { MaybeByteArray(None) }

    /// Creates [`MaybeByteArray`] assigning a value to it
    #[inline]
    pub fn some(val: ByteArray) -> MaybeByteArray { MaybeByteArray(Some(val)) }

    /// Transforms internal value layout returning whether this was possible without discarding any
    /// bit information
    #[inline]
    pub fn reshape(&mut self, to: u16) -> bool {
        match self.0 {
            None => true,
            Some(ref mut val) => val.reshape(to),
        }
    }
}

impl From<ByteArray> for MaybeByteArray {
    fn from(val: ByteArray) -> Self { MaybeByteArray(Some(val)) }
}

impl From<&ByteArray> for MaybeByteArray {
    fn from(val: &ByteArray) -> Self { MaybeByteArray(Some(*val)) }
}

impl From<&Option<ByteArray>> for MaybeByteArray {
    fn from(val: &Option<ByteArray>) -> Self { MaybeByteArray(*val) }
}

impl From<Option<&ByteArray>> for MaybeByteArray {
    fn from(val: Option<&ByteArray>) -> Self { MaybeByteArray(val.copied()) }
}

impl From<MaybeByteArray> for Option<ByteArray> {
    fn from(val: MaybeByteArray) -> Self { val.0 }
}

impl Deref for MaybeByteArray {
    type Target = Option<ByteArray>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

/// Type holding ByteArray of any layout
#[derive(Copy, Clone)]
pub struct ByteArray {
    /// Internal ByteArray representation, up to the possible maximum size of any supported
    /// ByteArray layout
    bytes: [u8; 1024],

    /// ByteArray layout used by the value
    length: u16,
}

impl Hash for ByteArray {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let clean = self.to_clean();
        clean.length.hash(state);
        state.write(&clean.bytes);
    }
}

impl Default for ByteArray {
    fn default() -> ByteArray { ByteArray { length: 1, bytes: [0u8; 1024] } }
}

impl AsRef<[u8]> for ByteArray {
    fn as_ref(&self) -> &[u8] { &self[..] }
}

impl AsMut<[u8]> for ByteArray {
    fn as_mut(&mut self) -> &mut [u8] { &mut self[..] }
}

impl Index<u16> for ByteArray {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        assert!(index < self.len());
        &self.bytes[index as usize]
    }
}

impl IndexMut<u16> for ByteArray {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        assert!(index < self.len());
        &mut self.bytes[index as usize]
    }
}

impl Index<RangeFull> for ByteArray {
    type Output = [u8];

    fn index(&self, _: RangeFull) -> &Self::Output { &self.bytes[..self.len() as usize] }
}

impl IndexMut<RangeFull> for ByteArray {
    fn index_mut(&mut self, _: RangeFull) -> &mut Self::Output {
        let len = self.len() as usize;
        &mut self.bytes[..len]
    }
}

impl Index<Range<u16>> for ByteArray {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &Self::Output {
        assert!(index.start < self.len() && index.end <= self.len());
        &self.bytes[index.start as usize..index.end as usize]
    }
}

impl IndexMut<Range<u16>> for ByteArray {
    fn index_mut(&mut self, index: Range<u16>) -> &mut Self::Output {
        assert!(index.start < self.len() && index.end <= self.len());
        &mut self.bytes[index.start as usize..index.end as usize]
    }
}

impl Index<RangeInclusive<u16>> for ByteArray {
    type Output = [u8];

    fn index(&self, index: RangeInclusive<u16>) -> &Self::Output {
        assert!(*index.start() < self.len() && *index.end() < self.len());
        &self.bytes[*index.start() as usize..*index.end() as usize]
    }
}

impl IndexMut<RangeInclusive<u16>> for ByteArray {
    fn index_mut(&mut self, index: RangeInclusive<u16>) -> &mut Self::Output {
        &mut self.bytes[*index.start() as usize..*index.end() as usize]
    }
}

impl Index<RangeFrom<u16>> for ByteArray {
    type Output = [u8];

    fn index(&self, index: RangeFrom<u16>) -> &Self::Output {
        assert!(index.start < self.len());
        &self.bytes[index.start as usize..self.len() as usize]
    }
}

impl IndexMut<RangeFrom<u16>> for ByteArray {
    fn index_mut(&mut self, index: RangeFrom<u16>) -> &mut Self::Output {
        assert!(index.start < self.len());
        let len = self.len() as usize;
        &mut self.bytes[index.start as usize..len]
    }
}

impl Index<RangeTo<u16>> for ByteArray {
    type Output = [u8];

    fn index(&self, index: RangeTo<u16>) -> &Self::Output {
        assert!(index.end <= self.len());
        &self.bytes[..index.end as usize]
    }
}

impl IndexMut<RangeTo<u16>> for ByteArray {
    fn index_mut(&mut self, index: RangeTo<u16>) -> &mut Self::Output {
        assert!(index.end <= self.len());
        &mut self.bytes[..index.end as usize]
    }
}

impl Index<RangeToInclusive<u16>> for ByteArray {
    type Output = [u8];

    fn index(&self, index: RangeToInclusive<u16>) -> &Self::Output {
        assert!(index.end < self.len());
        &self.bytes[..=index.end as usize]
    }
}

impl IndexMut<RangeToInclusive<u16>> for ByteArray {
    fn index_mut(&mut self, index: RangeToInclusive<u16>) -> &mut Self::Output {
        assert!(index.end < self.len());
        &mut self.bytes[..=index.end as usize]
    }
}

impl ByteArray {
    /// Creates zero value with a given layout
    #[inline]
    pub fn zero(length: u16) -> ByteArray {
        assert!(length <= 1024);
        ByteArray { length, bytes: [0u8; 1024] }
    }

    /// Creates value with the specified bit masked
    #[inline]
    pub fn masked_bit(bit_no: u16, length: u16) -> ByteArray {
        let mut zero = ByteArray { length, bytes: [0u8; 1024] };
        zero.bytes[(bit_no / 8) as usize] = 1 << (bit_no % 8);
        zero
    }

    /// Constructs ByteArray representation from a slice and a given layout.
    ///
    /// Fails returning `None` if the length of slice does not match the required layout byte
    /// length.
    pub fn with(slice: impl AsRef<[u8]>, length: u16) -> Option<ByteArray> {
        let slice = slice.as_ref();
        if slice.len() != length as usize {
            return None;
        }
        let mut me = ByteArray::from_slice(slice);
        me.length = length;
        Some(me)
    }

    /// Constructs value from slice of bytes.
    ///
    /// Panics if the length of the slice is greater than 1024 bytes.
    pub fn from_slice(slice: impl AsRef<[u8]>) -> ByteArray {
        let length = slice.as_ref().len();
        let mut bytes = [0u8; 1024];
        bytes[0..length].copy_from_slice(slice.as_ref());
        ByteArray { length: length as u16, bytes }
    }

    /// Constructs value from hex string
    #[cfg(feature = "std")]
    pub fn from_hex(s: &str) -> Result<ByteArray, amplify::hex::Error> {
        use amplify::hex::FromHex;
        let s = s.trim_start_matches("0x");
        let len = s.len() / 2;
        if len > 1024 {
            return Err(amplify::hex::Error::InvalidLength(1024, len));
        }
        let mut bytes = [0u8; 1024];
        let hex = Vec::<u8>::from_hex(s)?;
        bytes[0..len].copy_from_slice(&hex);
        Ok(ByteArray { length: hex.len() as u16, bytes })
    }

    /// Serializes value in hexadecimal format to a string
    #[cfg(feature = "std")]
    pub fn to_hex(self) -> String {
        let mut ret = String::with_capacity(2usize * self.len() as usize + 2);
        write!(ret, "0x").expect("writing to string");
        for ch in &self.bytes {
            write!(ret, "{:02x}", ch).expect("writing to string");
        }
        ret
    }

    /// Returns length of the used portion of the value
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u16 { self.length }

    /// Returns the ByteArray of zeros in the binary representation of `self`.
    #[inline]
    pub fn count_zeros(&self) -> u16 { self.len() - self.count_ones() }

    /// Returns the ByteArray of ones in the binary representation of `self`.
    pub fn count_ones(&self) -> u16 {
        let mut count = 0u16;
        for byte in &self[..] {
            count += byte.count_ones() as u16;
        }
        count
    }

    /// Measures minimum ByteArray of bits required to store the ByteArray.
    pub fn min_bit_len(&self) -> u16 {
        if self.len() == 0 {
            return 0;
        }
        let empty_bytes = self[..].iter().rev().take_while(|&&v| v == 0).count() as u16;
        let index = if self.len() > empty_bytes { self.len() - empty_bytes - 1 } else { 0 };
        let head_bits = 8 - self[index].leading_zeros();
        index * 8 + head_bits as u16
    }

    /// Detects if the value is equal to zero
    pub fn is_zero(self) -> bool {
        let clean = self.to_clean();
        clean.bytes == [0; 1024]
    }

    /// Ensures that all non-value bits are set to zero
    #[inline]
    pub fn clean(&mut self) {
        let len = self.len() as usize;
        self.bytes[len..].fill(0);
    }

    /// Returns a copy where all non-value bits are set to zero
    #[inline]
    pub fn to_clean(mut self) -> Self {
        self.clean();
        self
    }

    /// Transforms internal value layout returning whether this was possible without discarding any
    /// bit information
    pub fn reshape(&mut self, to: u16) -> bool {
        match (self.length, to) {
            (from, to) if from <= to => {
                self.length = to;
                true
            }
            (_, to) => {
                self.length = to;
                self.clean();
                false
            }
        }
    }

    /// Transforms internal value layout.
    ///
    /// # Returns
    /// Transformed ByteArray as an optional - or `None` if the operation was impossible without
    /// discarding bit information and `wrap` is set to false.
    pub fn reshaped(mut self, to: u16, wrap: bool) -> Option<ByteArray> {
        self.reshape(to).then(|| self).or(if wrap { Some(self) } else { None })
    }

    /// todo
    pub fn into_number(self, layout: Layout) -> (Number, bool) {
        let mut ret = Number::from(self.bytes);
        ret.reshape(layout);
        (ret, layout.bits() >= self.min_bit_len())
    }
}

/// Errors parsing literal values in AluVM assembly code
#[derive(Clone, Eq, PartialEq, Debug, Display, From)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(inner)]
#[non_exhaustive]
pub enum LiteralParseError {
    /// Error parsing decimal literal
    #[from]
    Int(core::num::ParseIntError),

    /// Error parsing float value
    #[from]
    #[display(Debug)]
    Float(amplify::num::apfloat::ParseError),

    /// Unknown literal
    #[display("unknown token `{0}` while parsing AluVM assembly literal")]
    UnknownLiteral(String),
}

impl Debug for ByteArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.length as usize;
        f.debug_struct("ByteArray")
            .field("length", &self.length)
            .field("bytes", {
                #[cfg(feature = "std")]
                {
                    use amplify::hex::ToHex;
                    &self.bytes[..len].to_hex()
                }
                #[cfg(not(feature = "std"))]
                {
                    &format!("{:#04X?}", &self.bytes[0..len])
                }
            })
            .finish()
    }
}

macro_rules! impl_byte_array_bytes_conv {
    ($len:literal) => {
        impl From<ByteArray> for [u8; $len] {
            fn from(val: ByteArray) -> Self {
                let len = (val.min_bit_len() + 7) as usize / 8;
                assert!(
                    len <= $len,
                    "attempt to convert ByteArray into a byte array with incorrect length",
                );
                let mut bytes = [0u8; $len];
                bytes[..len].copy_from_slice(&val.bytes[..len]);
                bytes
            }
        }

        impl From<[u8; $len]> for ByteArray {
            fn from(val: [u8; $len]) -> ByteArray {
                let mut bytes = [0u8; 1024];
                bytes[0..$len].copy_from_slice(&val[..]);
                ByteArray { length: $len, bytes }
            }
        }

        impl From<[u8; $len]> for MaybeByteArray {
            fn from(val: [u8; $len]) -> MaybeByteArray {
                MaybeByteArray::from(ByteArray::from(val))
            }
        }

        impl From<Option<[u8; $len]>> for MaybeByteArray {
            fn from(val: Option<[u8; $len]>) -> MaybeByteArray {
                MaybeByteArray::from(val.map(ByteArray::from))
            }
        }

        impl From<&Option<[u8; $len]>> for MaybeByteArray {
            fn from(val: &Option<[u8; $len]>) -> MaybeByteArray {
                MaybeByteArray::from(val.map(ByteArray::from))
            }
        }
    };
}

impl<T: ::core::convert::Into<MaybeByteArray>> From<StatusAnd<T>> for MaybeByteArray {
    fn from(init: StatusAnd<T>) -> Self {
        match init.status {
            Status::OK | Status::INEXACT => init.value.into(),
            _ => MaybeByteArray::none(),
        }
    }
}

impl_byte_array_bytes_conv!(1);
impl_byte_array_bytes_conv!(2);
impl_byte_array_bytes_conv!(4);
impl_byte_array_bytes_conv!(8);
impl_byte_array_bytes_conv!(16);
impl_byte_array_bytes_conv!(20);
impl_byte_array_bytes_conv!(32);
impl_byte_array_bytes_conv!(64);
impl_byte_array_bytes_conv!(128);
impl_byte_array_bytes_conv!(256);
impl_byte_array_bytes_conv!(512);
impl_byte_array_bytes_conv!(1024);
