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
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::string::FromUtf8Error;

use amplify::IoError;
use bitcoin_hashes::Hash;

use crate::data::encoding::DecodeError::InvalidBool;
use crate::data::{ByteStr, FloatLayout, IntLayout, Layout, MaybeNumber, Number, NumberLayout};
use crate::libs::{IsaSeg, IsaSegError, Lib, LibId, LibSeg, LibSegOverflow, LibSite, SegmentError};

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
    /// data writing error ({0})
    #[from]
    #[from(io::Error)]
    Io(IoError),

    /// string length is {0}, which exceeds 255 bytes limit
    StringTooLong(usize),

    /// collection contains {0} items, which exceeds [`u16::MAX`] limit
    ByteLimitExceeded(usize),

    /// collection contains {0} items, which exceeds [`u16::MAX`] limit
    WordLimitExceeded(usize),
}

/// Errors decoding AluVM data containers
#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum DecodeError {
    /// data reading error ({0})
    #[from]
    #[from(io::Error)]
    Io(IoError),

    /// invalid bool value `{0}`
    InvalidBool(u8),

    /// invalid UTF8 string data
    ///
    /// details: {0}
    #[from]
    InvalidUtf8(FromUtf8Error),

    /// number data does not match provided layout {0:?}
    ///
    /// number data: {1:#02x?}
    NumberLayout(Layout, Vec<u8>),

    /// unknown float layout type `{0}`
    FloatLayout(u8),

    /// Library construction errors
    #[display(inner)]
    #[from]
    Lib(SegmentError),

    /// Library segment construction error
    #[display(inner)]
    #[from]
    LibSeg(LibSegOverflow),

    /// ISAE segment construction error
    #[display(inner)]
    #[from]
    IsaSeg(IsaSegError),
}

/// Wrapper around collections which may contain at most [`u8::MAX`] elements
pub struct MaxLenByte<I, A = ()>(pub I, PhantomData<A>);

impl<I, A> MaxLenByte<I, A> {
    /// Constructs limited-size wrapper around the provided type
    pub fn new(iter: I) -> Self { Self(iter, Default::default()) }

    /// Releases inner type
    pub fn release(self) -> I { self.0 }
}

/// Wrapper around collections which may contain at most [`u16::MAX`] elements
pub struct MaxLenWord<I, A = ()>(pub I, PhantomData<A>);

impl<I, A> MaxLenWord<I, A> {
    /// Constructs limited-size wrapper around the provided type
    pub fn new(iter: I) -> Self { Self(iter, Default::default()) }

    /// Releases inner type
    pub fn release(self) -> I { self.0 }
}

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

impl Decode for bool {
    type Error = DecodeError;

    #[inline]
    fn decode(reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match u8::decode(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            invalid => Err(InvalidBool(invalid)),
        }
    }
}

impl Encode for u8 {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        writer.write_all(&[*self])?;
        Ok(1)
    }
}

impl Decode for u8 {
    type Error = io::Error;

    #[inline]
    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        Ok(byte[0])
    }
}

impl Encode for u16 {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        writer.write_all(&self.to_le_bytes())?;
        Ok(2)
    }
}

impl Decode for u16 {
    type Error = io::Error;

    #[inline]
    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut word = [0u8; 2];
        reader.read_exact(&mut word)?;
        Ok(u16::from_le_bytes(word))
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
        writer.write_all(self.as_bytes())?;
        Ok(len + 1)
    }
}

impl Decode for String {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = u8::decode(&mut reader)?;
        let mut s = vec![0u8; len as usize];
        reader.read_exact(&mut s)?;
        String::from_utf8(s).map_err(DecodeError::from)
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

impl Decode for Option<String> {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = u8::decode(&mut reader)?;
        if len == 0 {
            return Ok(None);
        }
        let mut s = vec![0u8; len as usize];
        reader.read_exact(&mut s)?;
        String::from_utf8(s).map_err(DecodeError::from).map(Some)
    }
}

impl<'i, I> Encode for MaxLenByte<&'i I>
where
    &'i I: IntoIterator,
    <&'i I as IntoIterator>::IntoIter: ExactSizeIterator,
    <&'i I as IntoIterator>::Item: Encode,
    EncodeError: From<<<&'i I as IntoIterator>::Item as Encode>::Error>,
{
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let iter = self.0.into_iter();
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

impl<T, I> Decode for MaxLenByte<T, I>
where
    T: FromIterator<I>,
    I: Decode,
    DecodeError: From<<I as Decode>::Error>,
{
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = u8::decode(&mut reader)?;
        let mut vec = vec![];
        for _ in 0..len {
            vec.push(I::decode(&mut reader)?);
        }
        Ok(MaxLenByte::new(vec.into_iter().collect()))
    }
}

impl<'i, I> Encode for MaxLenWord<&'i I>
where
    &'i I: IntoIterator,
    <&'i I as IntoIterator>::IntoIter: ExactSizeIterator,
    <&'i I as IntoIterator>::Item: Encode,
    EncodeError: From<<<&'i I as IntoIterator>::Item as Encode>::Error>,
{
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let iter = self.0.into_iter();
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

impl<T, I> Decode for MaxLenWord<T, I>
where
    T: FromIterator<I>,
    I: Decode,
    DecodeError: From<<I as Decode>::Error>,
{
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = u16::decode(&mut reader)?;
        let mut vec = vec![];
        for _ in 0..len {
            vec.push(I::decode(&mut reader)?);
        }
        Ok(MaxLenWord::new(vec.into_iter().collect()))
    }
}

impl<A, B> Encode for (A, B)
where
    A: Encode,
    B: Encode,
    EncodeError: From<A::Error> + From<B::Error>,
{
    type Error = EncodeError;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        Ok(self.0.encode(&mut writer)? + self.1.encode(&mut writer)?)
    }
}

impl<A, B> Decode for (A, B)
where
    A: Decode,
    B: Decode,
    DecodeError: From<A::Error> + From<B::Error>,
{
    type Error = DecodeError;

    #[inline]
    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok((A::decode(&mut reader)?, B::decode(&mut reader)?))
    }
}

impl Encode for ByteStr {
    type Error = io::Error;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let len = self.len();
        len.encode(&mut writer)?;
        writer.write_all(self.as_ref())?;
        Ok(len as usize + 2)
    }
}

impl Decode for ByteStr {
    type Error = io::Error;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = u16::decode(&mut reader)?;
        let mut vec = vec![0u8; len as usize];
        reader.read_exact(&mut vec)?;
        Ok(ByteStr::with(vec))
    }
}

impl Encode for Option<ByteStr> {
    type Error = io::Error;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        if let Some(s) = self {
            s.encode(writer)
        } else {
            Ok(0u8.encode(writer)?)
        }
    }
}

impl Decode for Option<ByteStr> {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let len = u8::decode(&mut reader)?;
        if len == 0 {
            return Ok(None);
        }
        let mut s = vec![0u8; len as usize];
        reader.read_exact(&mut s)?;
        Ok(Some(ByteStr::with(s)))
    }
}

impl Encode for Number {
    type Error = io::Error;

    #[inline]
    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let count = self.layout().encode(&mut writer)?;
        writer.write_all(self.as_ref())?;

        let len = self.len();
        Ok(count + len as usize)
    }
}

impl Decode for Number {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let layout = Layout::decode(&mut reader)?;
        let mut vec = vec![0u8; layout.bytes() as usize];
        reader.read_exact(&mut vec)?;
        Number::with(&vec, layout).ok_or(DecodeError::NumberLayout(layout, vec))
    }
}

impl Encode for MaybeNumber {
    type Error = io::Error;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        match **self {
            Some(number) => Ok(1u16.encode(&mut writer)? + number.encode(&mut writer)?),
            None => 0u16.encode(writer),
        }
    }
}

impl Decode for MaybeNumber {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match u8::decode(&mut reader)? {
            0 => Ok(MaybeNumber::none()),
            1 => Ok(Number::decode(reader)?.into()),
            unknown => Err(DecodeError::InvalidBool(unknown)),
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

impl Decode for IntLayout {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(IntLayout { signed: bool::decode(&mut reader)?, bytes: u16::decode(&mut reader)? })
    }
}

impl Encode for FloatLayout {
    type Error = io::Error;

    #[inline]
    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        (*self as u8).encode(writer)
    }
}

impl Decode for FloatLayout {
    type Error = DecodeError;

    fn decode(reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let val = u8::decode(reader)?;
        FloatLayout::with(val).ok_or(DecodeError::FloatLayout(val))
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

impl Decode for Layout {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::decode(&mut reader)? {
            i if i <= 1 => IntLayout { signed: i == 1, bytes: u16::decode(reader)? }.into(),
            float => FloatLayout::with(float).ok_or(DecodeError::FloatLayout(float))?.into(),
        })
    }
}

impl Encode for LibId {
    type Error = io::Error;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        let slice = self.into_inner();
        writer.write_all(&slice)?;
        Ok(slice.len())
    }
}

impl Decode for LibId {
    type Error = io::Error;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut slice = [0u8; LibId::LEN];
        reader.read_exact(&mut slice)?;
        Ok(LibId::from_inner(slice))
    }
}

impl Encode for IsaSeg {
    type Error = EncodeError;

    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        self.to_string().encode(writer)
    }
}

impl Decode for IsaSeg {
    type Error = DecodeError;

    fn decode(reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let s = String::decode(reader)?;
        IsaSeg::with(s).map_err(DecodeError::from)
    }
}

impl Encode for LibSeg {
    type Error = EncodeError;

    fn encode(&self, writer: impl Write) -> Result<usize, Self::Error> {
        MaxLenByte::new(self).encode(writer)
    }
}

impl Decode for LibSeg {
    type Error = DecodeError;

    fn decode(reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let seg: Vec<_> = MaxLenByte::decode(reader)?.release();
        Ok(LibSeg::from_iter(seg)?)
    }
}

impl Encode for LibSite {
    type Error = io::Error;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        Ok(self.lib.encode(&mut writer)? + self.pos.encode(&mut writer)?)
    }
}

impl Decode for LibSite {
    type Error = io::Error;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let id = LibId::decode(&mut reader)?;
        let pos = u16::decode(&mut reader)?;
        Ok(LibSite::with(pos, id))
    }
}

impl Encode for Lib {
    type Error = EncodeError;

    fn encode(&self, mut writer: impl Write) -> Result<usize, Self::Error> {
        Ok(self.isae_segment().encode(&mut writer)?
            + self.code.encode(&mut writer)?
            + self.data.encode(&mut writer)?
            + self.libs.encode(&mut writer)?)
    }
}

impl Decode for Lib {
    type Error = DecodeError;

    fn decode(mut reader: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Lib::with(
            String::decode(&mut reader)?.as_str(),
            ByteStr::decode(&mut reader)?.to_vec(),
            ByteStr::decode(&mut reader)?.to_vec(),
            LibSeg::decode(&mut reader)?,
        )?)
    }
}
