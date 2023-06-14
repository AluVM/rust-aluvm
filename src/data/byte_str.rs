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

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::convert::TryFrom;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::Range;

use amplify::confinement::{SmallBlob, TinyBlob};
use amplify::num::error::OverflowError;

/// Large binary bytestring object.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteStr {
    /// Adjusted slice length.
    len: u16,

    /// Slice bytes
    #[doc(hidden)]
    pub bytes: Box<[u8; u16::MAX as usize]>,
}

impl Default for ByteStr {
    fn default() -> ByteStr { ByteStr { len: 0, bytes: Box::new([0u8; u16::MAX as usize]) } }
}

impl AsRef<[u8]> for ByteStr {
    #[inline]
    fn as_ref(&self) -> &[u8] { &self.bytes[..self.len as usize] }
}

impl AsMut<[u8]> for ByteStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] { &mut self.bytes[..self.len as usize] }
}

impl Borrow<[u8]> for ByteStr {
    #[inline]
    fn borrow(&self) -> &[u8] { &self.bytes[..self.len as usize] }
}

impl BorrowMut<[u8]> for ByteStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] { &mut self.bytes[..self.len as usize] }
}

impl Extend<u8> for ByteStr {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        let mut pos = self.len();
        let iter = iter.into_iter();
        for byte in iter {
            assert!(pos < u16::MAX);
            self.bytes[pos as usize] = byte;
            pos += 1;
        }
        self.len = pos;
    }
}

impl From<&TinyBlob> for ByteStr {
    fn from(blob: &TinyBlob) -> Self {
        let len = blob.len_u8() as u16;
        let mut bytes = [0u8; u16::MAX as usize];
        bytes[0..(len as usize)].copy_from_slice(blob.as_slice());
        ByteStr { len, bytes: Box::new(bytes) }
    }
}

impl From<&SmallBlob> for ByteStr {
    fn from(blob: &SmallBlob) -> Self {
        let len = blob.len_u16();
        let mut bytes = [0u8; u16::MAX as usize];
        bytes[0..(len as usize)].copy_from_slice(blob.as_slice());
        ByteStr { len, bytes: Box::new(bytes) }
    }
}

impl From<TinyBlob> for ByteStr {
    fn from(blob: TinyBlob) -> Self { ByteStr::from(&blob) }
}

impl From<SmallBlob> for ByteStr {
    fn from(blob: SmallBlob) -> Self { ByteStr::from(&blob) }
}

impl TryFrom<&[u8]> for ByteStr {
    type Error = OverflowError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let len = slice.len();
        if len > u16::MAX as usize {
            return Err(OverflowError { max: u16::MAX as usize + 1, value: len });
        }
        let mut bytes = [0u8; u16::MAX as usize];
        bytes[0..len].copy_from_slice(slice.as_ref());
        Ok(ByteStr { len: len as u16, bytes: Box::new(bytes) })
    }
}

impl ByteStr {
    /// Constructs blob from slice of bytes.
    ///
    /// Panics if the length of the slice is greater than `u16::MAX` bytes.
    #[inline]
    pub fn with(slice: impl AsRef<[u8]>) -> ByteStr {
        ByteStr::try_from(slice.as_ref())
            .expect("internal error: ByteStr::with requires slice <= u16::MAX + 1")
    }

    /// Returns correct length of the string, in range `0 ..= u16::MAX`
    #[inline]
    pub fn len(&self) -> u16 { self.len }

    /// Returns when the string has a zero length
    #[inline]
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// Adjusts the length of the string
    #[inline]
    pub fn adjust_len(&mut self, new_len: u16) { self.len = new_len }

    /// Extends the length of the string if necessary
    #[inline]
    pub fn extend_len(&mut self, new_len: u16) { self.len = new_len.max(self.len) }

    /// Fills range within a string with the provided byte value, increasing string length if
    /// necessary
    pub fn fill(&mut self, range: Range<u16>, val: u8) {
        let start = range.start;
        let end = range.end;
        self.extend_len(end);
        self.bytes[start as usize..end as usize].fill(val);
    }

    /// Returns vector representation of the contained bytecode
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> { self.as_ref().to_vec() }
}

#[cfg(not(feature = "std"))]
impl Debug for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{:#04X?}", self.as_ref()) }
}

#[cfg(feature = "std")]
impl Debug for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;

        f.debug_tuple("ByteStr").field(&self.as_ref().to_hex()).finish()
    }
}

#[cfg(feature = "std")]
impl Display for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        use amplify::hex::ToHex;

        let vec = Vec::from(&self.bytes[..self.len as usize]);
        if f.alternate() {
            for (line, slice) in self.as_ref().chunks(16).enumerate() {
                write!(f, "\x1B[0;35m{:>1$x}0  |  \x1B[0m", line, f.width().unwrap_or(1) - 1)?;
                for (pos, byte) in slice.iter().enumerate() {
                    write!(f, "{:02x} ", byte)?;
                    if pos == 7 {
                        f.write_char(' ')?;
                    }
                }
                if slice.len() < 8 {
                    f.write_char(' ')?;
                }
                write!(
                    f,
                    "{:1$}\x1B[0;35m|\x1B[0m  ",
                    ' ',
                    16usize.saturating_sub(slice.len()) * 3 + 1
                )?;
                for byte in slice {
                    f.write_str(&if byte.is_ascii_control()
                        || byte.is_ascii_whitespace()
                        || !byte.is_ascii()
                    {
                        s!("\x1B[0;35m·\x1B[0m")
                    } else {
                        String::from(char::from(*byte))
                    })?;
                }
                f.write_char('\n')?;
            }
            Ok(())
            // write!(f, "{}..{}", self.bytes[..4].to_hex(), self.bytes[(self.len() -
            // 4)..].to_hex())
        } else if let Ok(s) = String::from_utf8(vec) {
            f.write_str("\"")?;
            f.write_str(&s)?;
            f.write_str("\"")
        } else {
            f.write_str(&self.as_ref().to_hex())
        }
    }
}

#[cfg(not(feature = "std"))]
impl Display for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{:#04X?}", self.as_ref()) }
}

/*
#[cfg(feature = "strict_encoding")]
mod _strict_encoding {
    use std::convert::TryFrom;
    use std::io::{Read, Write};
    use std::ops::Deref;

    use strict_encoding::{StrictDecode, StrictEncode};

    use super::ByteStr;

    impl StrictEncode for ByteStr {
        fn strict_encode<E: Write>(&self, e: E) -> Result<usize, strict_encoding::Error> {
            self.as_ref().strict_encode(e)
        }
    }

    impl StrictDecode for ByteStr {
        fn strict_decode<D: Read>(d: D) -> Result<Self, strict_encoding::Error> {
            let data = Vec::<u8>::strict_decode(d)?;
            Ok(ByteStr::try_from(data.deref()).expect("strict encoding can't read more than 67 kb"))
        }
    }
}
 */

#[cfg(feature = "serde")]
mod _serde {
    use std::convert::TryFrom;
    use std::ops::Deref;

    use serde_crate::de::Error;
    use serde_crate::{Deserialize, Deserializer, Serialize, Serializer};

    use super::ByteStr;

    impl Serialize for ByteStr {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.as_ref().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for ByteStr {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let vec = Vec::<u8>::deserialize(deserializer)?;
            ByteStr::try_from(vec.deref())
                .map_err(|_| D::Error::invalid_length(vec.len(), &"max u16::MAX bytes"))
        }
    }
}
