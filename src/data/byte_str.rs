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
use core::fmt::{self, Display, Formatter};
use core::ops::RangeInclusive;

/// Large binary bytestring object.
///
/// NB: Since byte string length is expressed with `u16` integer, it is 0-based, i.e. one character
/// string has length of `0`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ByteStr {
    /// Adjusted slice length.
    ///
    /// Values from `0` to `u16:MAX` represent string length (with `0` meaning "no value is
    /// stored").
    ///
    /// `None` value indicates that the data occupy full length (i.e. `u16::MAX + 1`).
    len: Option<u16>,

    /// Slice bytes
    pub bytes: Box<[u8; u16::MAX as usize]>,
}

impl Default for ByteStr {
    fn default() -> ByteStr { ByteStr { len: Some(0), bytes: Box::new([0u8; u16::MAX as usize]) } }
}

impl AsRef<[u8]> for ByteStr {
    fn as_ref(&self) -> &[u8] { &self.bytes[..self.len()] }
}

impl AsMut<[u8]> for ByteStr {
    fn as_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        &mut self.bytes[..len]
    }
}

impl ByteStr {
    /// Constructs blob from slice of bytes.
    ///
    /// Panics if the length of the slice is greater than `u16::MAX` bytes.
    pub fn with(slice: impl AsRef<[u8]>) -> ByteStr {
        let len = slice.as_ref().len();
        assert!(len <= u16::MAX as usize + 1);
        let mut bytes = [0u8; u16::MAX as usize];
        bytes[0..len].copy_from_slice(slice.as_ref());
        ByteStr {
            len: if len > u16::MAX as usize { None } else { Some(len as u16) },
            bytes: Box::new(bytes),
        }
    }

    /// Returns correct length of the string, in range `0 ..= u16::MAX + 1`
    #[inline]
    pub fn len(&self) -> usize { self.len.map(|len| len as usize).unwrap_or(u16::MAX as usize + 1) }

    /// Returns when the string has a zero length
    #[inline]
    pub fn is_empty(&self) -> bool { self.len == Some(0) }

    /// Adjusts the length of the string if necessary
    #[inline]
    pub fn adjust_len(&mut self, new_len: u16, inclusive: bool) {
        match (self.len, new_len, inclusive) {
            (Some(_), u16::MAX, true) => self.len = None,
            (Some(len), new, true) if len <= new => self.len = Some(new + 1),
            (Some(len), new, false) if len < new => self.len = Some(new),
            _ => {}
        }
    }

    /// Fills range within a string with the provided byte value, increasing string length if
    /// necessary
    pub fn fill(&mut self, range: RangeInclusive<u16>, val: u8) {
        let start = *range.start();
        let end = *range.end();
        self.adjust_len(end, true);
        self.bytes[start as usize..=end as usize].fill(val);
    }
}

#[cfg(feature = "std")]
impl Display for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        let vec = Vec::from(&self.bytes[..self.len()]);
        if let Ok(s) = String::from_utf8(vec) {
            f.write_str("\"")?;
            f.write_str(&s)?;
            f.write_str("\"")
        } else if f.alternate() && self.len() > 4 {
            write!(f, "{}..{}", self.bytes[..4].to_hex(), self.bytes[(self.len() - 4)..].to_hex())
        } else {
            f.write_str(&self.bytes[0usize..(self.len())].to_hex())
        }
    }
}

#[cfg(not(feature = "std"))]
impl Display for ByteStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#04X?}", &self.bytes[0usize..(self.len())])
    }
}
