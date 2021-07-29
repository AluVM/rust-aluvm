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

//! Helper traits and default implementations for encoding elements of AliVM container types

use std::io::{self, Read, Write};

use amplify::IoError;
use bitcoin_hashes::Hash;

use crate::data::{ByteStr, FloatLayout, IntLayout, Layout, MaybeNumber, Number};
use crate::isa::InstructionSet;
use crate::libs::{Lib, LibId, LibSeg};

/// Trait for encodable container data structures used by AluVM and runtime environments
pub trait Encode {
    /// Type-specific encoding error enumeration
    type Error: std::error::Error + From<io::Error>;

    /// Encodes data structure to a writer
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error>;

    /// Serializes data structure as an in-memory array
    #[inline]
    fn serialize(&self) -> Vec<u8> {
        let mut wrter = vec![];
        self.encode(&mut wrter).expect("in-memory encoding");
        wrter
    }
}

/// Trait for container data structures which can be read or deserialized
pub trait Decode {
    /// Type-specific decoding error enumeration
    type Error: std::error::Error + From<io::Error>;

    /// Decodes data structure from a reader
    fn decode(reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Deserializes data structure from given byte slice
    #[inline]
    fn deserialize(from: impl AsRef<[u8]>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::decode(from.as_ref())
    }
}

/// Errors encoding AluVM data containers
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum EncodeError {
    /// data reading error ({0})
    #[from]
    #[from(io::Error)]
    Io(IoError),

    /// string length is {0}, which exceeds 255 bytes limit
    StringTooLong(usize),

    /// collection contains {0} items, which exceeds [`u18::MAX`] limit
    ByteLimitExceeded(usize),

    /// collection contains {0} items, which exceeds [`u16::MAX`] limit
    WordLimitExceeded(usize),
}

/// Wrapper around collections which may contain at most [`u8::MAX`] elements
pub struct MaxLenByte<'i, I>(pub &'i I)
where
    &'i I: IntoIterator;

/// Wrapper around collections which may contain at most [`u16::MAX`] elements
pub struct MaxLenWord<'i, I>(pub &'i I)
where
    &'i I: IntoIterator;

impl<T> Encode for &T
where
    T: Encode,
{
    type Error = T::Error;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> { (*self).encode(writer) }
}

impl Encode for bool {
    type Error = io::Error;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        (*self as u8).encode(writer)
    }
}

impl Encode for u8 {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        writer.write(&[*self])?;
        Ok(1)
    }
}

impl Encode for u16 {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        writer.write(&self.to_le_bytes())?;
        Ok(2)
    }
}

impl Encode for String {
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let len = self.as_bytes().len();
        if len > u8::MAX as usize {
            return Err(EncodeError::StringTooLong(len));
        }
        (len as u8).encode(&mut writer)?;
        writer.write(self.as_bytes())?;
        Ok(len + 1)
    }
}

impl Encode for Option<String> {
    type Error = EncodeError;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        if let Some(s) = self {
            s.encode(writer)
        } else {
            Ok(0u8.encode(writer)?)
        }
    }
}

impl<'i, I> Encode for MaxLenByte<'i, I>
where
    &'i I: IntoIterator,
    <&'i I as IntoIterator>::IntoIter: ExactSizeIterator,
    <&'i I as IntoIterator>::Item: Encode,
    EncodeError: From<<<&'i I as IntoIterator>::Item as Encode>::Error>,
{
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let iter = (&self.0).into_iter();
        let len = iter.len();
        if len > u8::MAX as usize {
            return Err(EncodeError::ByteLimitExceeded(len));
        }
        (len as u8).encode(&mut writer)?;
        let mut count = 1;
        for item in iter {
            count += item.encode(&mut writer)?;
        }
        Ok(count)
    }
}

impl<'i, I> Encode for MaxLenWord<'i, I>
where
    &'i I: IntoIterator,
    <&'i I as IntoIterator>::IntoIter: ExactSizeIterator,
    <&'i I as IntoIterator>::Item: Encode,
    EncodeError: From<<<&'i I as IntoIterator>::Item as Encode>::Error>,
{
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let iter = (&self.0).into_iter();
        let len = iter.len();
        if len > u16::MAX as usize {
            return Err(EncodeError::WordLimitExceeded(len));
        }
        (len as u16).encode(&mut writer)?;
        let mut count = 2;
        for item in iter {
            count += item.encode(&mut writer)?;
        }
        Ok(count)
    }
}

impl Encode for ByteStr {
    type Error = io::Error;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let len = self.len();
        len.encode(&mut writer)?;
        writer.write(self.as_ref())?;
        Ok(len as usize + 2)
    }
}

impl Encode for Number {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let len = self.len();
        writer.write(&len.to_le_bytes())?;
        writer.write(self.as_ref())?;
        Ok(len as usize + 2)
    }
}

impl Encode for MaybeNumber {
    type Error = io::Error;

    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        match **self {
            Some(number) => number.encode(writer),
            None => 0u16.encode(writer),
        }
    }
}

impl Encode for IntLayout {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        Ok(self.signed.encode(&mut writer)? + self.bytes.encode(&mut writer)?)
    }
}

impl Encode for FloatLayout {
    type Error = io::Error;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        (*self as u8).encode(writer)
    }
}

impl Encode for Layout {
    type Error = io::Error;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        match self {
            Layout::Integer(layout) => layout.encode(writer),
            Layout::Float(layout) => layout.encode(writer),
        }
    }
}

impl Encode for LibId {
    type Error = io::Error;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let slice = self.into_inner();
        writer.write(&slice)?;
        Ok(slice.len())
    }
}

impl Encode for LibSeg {
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        MaxLenByte(self).encode(&mut writer)
    }
}

impl<E> Encode for Lib<E>
where
    E: InstructionSet,
{
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        Ok(self.isae_segment().encode(&mut writer)?
            + self.code_segment.encode(&mut writer)?
            + self.data_segment.encode(&mut writer)?
            + self.libs_segment.encode(&mut writer)?)
    }
}
