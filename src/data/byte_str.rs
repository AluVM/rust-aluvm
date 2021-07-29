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

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter, Write};
use core::ops::Range;

use amplify::num::error::OverflowError;

/// Large binary bytestring object.
///
/// NB: Since byte string length is expressed with `u16` integer, it is 0-based, i.e. one character
/// string has length of `0`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
        let mut iter = iter.into_iter();
        while let Some(byte) = iter.next() {
            assert!(pos < u16::MAX);
            self.bytes[pos as usize] = byte;
            pos += 1;
        }
        self.len = pos as u16;
    }
}

impl TryFrom<&[u8]> for ByteStr {
    type Error = OverflowError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let len = slice.as_ref().len();
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

    /// Adjusts the length of the string if necessary
    #[inline]
    pub fn adjust_len(&mut self, new_len: u16) { self.len = new_len }

    /// Fills range within a string with the provided byte value, increasing string length if
    /// necessary
    pub fn fill(&mut self, range: Range<u16>, val: u8) {
        let start = range.start;
        let end = range.end;
        self.adjust_len(end);
        self.bytes[start as usize..end as usize].fill(val);
    }

    /// Returns vector representation of the contained bytecode
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> { self.as_ref().to_vec() }
}

#[cfg(feature = "std")]
impl Display for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        let vec = Vec::from(&self.bytes[..self.len as usize]);
        if let Ok(s) = String::from_utf8(vec) {
            f.write_str("\"")?;
            f.write_str(&s)?;
            f.write_str("\"")
        } else if f.alternate() && self.len() > 4 {
            for (line, slice) in self.as_ref().chunks(16).enumerate() {
                write!(f, "  \x1B[1;34m{:>5x}0  |  \x1B[0m", line)?;
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
                    "{:1$}\x1B[1;34m|\x1B[0m  ",
                    ' ',
                    16usize.saturating_sub(slice.len()) * 3 + 1
                )?;
                for byte in slice {
                    f.write_str(&if byte.is_ascii_control()
                        || byte.is_ascii_whitespace()
                        || !byte.is_ascii()
                    {
                        s!("\x1B[5;38;240m·\x1B[0m")
                    } else {
                        String::from(char::from(*byte))
                    })?;
                }
                f.write_char('\n')?;
            }
            Ok(())
            // write!(f, "{}..{}", self.bytes[..4].to_hex(), self.bytes[(self.len() -
            // 4)..].to_hex())
        } else {
            f.write_str(&self.as_ref().to_hex())
        }
    }
}

#[cfg(not(feature = "std"))]
impl Display for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#04X?}", &self.bytes[0usize..self.len as usize])
    }
}
