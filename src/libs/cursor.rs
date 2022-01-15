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

use core::convert::TryInto;
#[cfg(feature = "std")]
use core::fmt::{self, Debug, Display, Formatter};

use amplify::num::{u1, u2, u24, u3, u4, u5, u6, u7};

use super::{CodeEofError, LibId, LibSeg, Read, Write, WriteError};
use crate::data::Number;
use crate::isa::{Bytecode, Instr, InstructionSet};
use crate::libs::constants::{CODE_SEGMENT_MAX_LEN, DATA_SEGMENT_MAX_LEN};
use crate::reg::NumericRegister;

/// Cursor for accessing bytecode bounded by [`CODE_SEGMENT_MAX_LEN`] length and data segment
/// bounded by [`DATA_SEGMENT_MAX_LEN`]
pub struct Cursor<'a, T, D>
where
    T: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    bytecode: T,
    bit_pos: u3,
    byte_pos: u16,
    data: D,
    libs: &'a LibSeg,
}

#[cfg(feature = "std")]
impl<'a, T, D> Debug for Cursor<'a, T, D>
where
    T: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        f.debug_struct("Cursor")
            .field("bytecode", &self.as_ref().to_hex())
            .field("byte_pos", &self.byte_pos)
            .field("bit_pos", &self.bit_pos)
            .field("data", &self.data.as_ref().to_hex())
            .field("libs", &self.libs)
            .finish()
    }
}

#[cfg(feature = "std")]
impl<'a, T, D> Display for Cursor<'a, T, D>
where
    T: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        write!(f, "{}:{} @ ", self.byte_pos, self.bit_pos)?;
        let hex = self.as_ref().to_hex();
        if f.alternate() {
            write!(f, "{}..{}", &hex[..4], &hex[hex.len() - 4..])
        } else {
            f.write_str(&hex)
        }
    }
}

impl<'a, T, D> Cursor<'a, T, D>
where
    T: AsRef<[u8]>,
    D: AsRef<[u8]> + Default,
    Self: 'a,
{
    /// Creates new cursor able to write the bytecode and data, using provided immutable libs
    /// segment
    #[inline]
    pub fn new(bytecode: T, libs: &'a LibSeg) -> Cursor<'a, T, D> {
        Cursor { bytecode, byte_pos: 0, bit_pos: u3::MIN, data: D::default(), libs }
    }
}

impl<'a, T, D> Cursor<'a, T, D>
where
    T: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    /// Creates cursor from the provided byte string utilizing existing libs segment
    ///
    /// # Panics
    ///
    /// If the length of the bytecode exceeds [`CODE_SEGMENT_MAX_LEN`] or length of the data
    /// [`DATA_SEGMENT_MAX_LEN`]
    #[inline]
    pub fn with(bytecode: T, data: D, libs: &'a LibSeg) -> Cursor<'a, T, D> {
        assert!(bytecode.as_ref().len() <= CODE_SEGMENT_MAX_LEN);
        assert!(data.as_ref().len() <= DATA_SEGMENT_MAX_LEN);
        Cursor { bytecode, byte_pos: 0, bit_pos: u3::MIN, data, libs }
    }

    /// Converts writer into data segment
    #[inline]
    pub fn into_data_segment(self) -> D { self.data }

    #[inline]
    fn as_ref(&self) -> &[u8] { self.bytecode.as_ref() }

    fn extract(&mut self, bit_count: u3) -> Result<u8, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let byte = self.as_ref()[self.byte_pos as usize];
        let mut mask = 0x00u8;
        let mut cnt = bit_count.as_u8();
        while cnt > 0 {
            mask <<= 1;
            mask |= 0x01;
            cnt -= 1;
        }
        mask <<= self.bit_pos.as_u8();
        let val = (byte & mask) >> self.bit_pos.as_u8();
        self.inc_bits(bit_count).map(|_| val)
    }

    fn inc_bits(&mut self, bit_count: u3) -> Result<(), CodeEofError> {
        let pos = self.bit_pos.as_u8() + bit_count.as_u8();
        self.bit_pos = u3::with(pos % 8);
        self._inc_bytes_inner(pos as u16 / 8)
    }

    fn inc_bytes(&mut self, byte_count: u16) -> Result<(), CodeEofError> {
        assert_eq!(
            self.bit_pos.as_u8(),
            0,
            "attempt to access (multiple) bytes at a non-byte aligned position"
        );
        self._inc_bytes_inner(byte_count)
    }

    #[inline]
    fn _inc_bytes_inner(&mut self, byte_count: u16) -> Result<(), CodeEofError> {
        self.byte_pos = self.byte_pos.checked_add(byte_count).ok_or(CodeEofError)?;
        Ok(())
    }
}

impl<'a, T, D> Cursor<'a, T, D>
where
    T: AsRef<[u8]> + AsMut<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    fn as_mut(&mut self) -> &mut [u8] { self.bytecode.as_mut() }
}

impl<'a, T, D> Cursor<'a, T, D>
where
    T: AsRef<[u8]> + AsMut<[u8]>,
    D: AsRef<[u8]> + AsMut<[u8]> + Extend<u8>,
    Self: 'a,
{
    fn write_unique(&mut self, bytes: &[u8]) -> Result<u16, WriteError> {
        // We write the value only if the value is not yet present in the data segment
        let len = bytes.len();
        let offset = self.data.as_ref().len();
        if let Some(offset) = self.data.as_ref().windows(len).position(|window| window == bytes) {
            Ok(offset as u16)
        } else if offset + len > DATA_SEGMENT_MAX_LEN {
            Err(WriteError::DataNotFittingSegment)
        } else {
            self.data.extend(bytes.iter().copied());
            Ok(offset as u16)
        }
    }
}

impl<'a, T, D> Read for Cursor<'a, T, D>
where
    T: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    #[inline]
    fn is_eof(&self) -> bool { self.byte_pos as usize >= self.as_ref().len() }

    #[inline]
    fn pos(&self) -> u16 { self.byte_pos }

    #[inline]
    fn seek(&mut self, byte_pos: u16) -> Result<u16, CodeEofError> {
        if byte_pos as usize >= self.as_ref().len() {
            return Err(CodeEofError);
        }
        let old_pos = self.byte_pos;
        self.byte_pos = byte_pos;
        Ok(old_pos)
    }

    fn peek_u8(&self) -> Result<u8, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        Ok(self.as_ref()[self.byte_pos as usize])
    }

    fn read_bool(&mut self) -> Result<bool, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let byte = self.extract(u3::with(1))?;
        Ok(byte == 0x01)
    }

    fn read_u1(&mut self) -> Result<u1, CodeEofError> {
        Ok(self.extract(u3::with(1))?.try_into().expect("bit extractor failure"))
    }

    fn read_u2(&mut self) -> Result<u2, CodeEofError> {
        Ok(self.extract(u3::with(2))?.try_into().expect("bit extractor failure"))
    }

    fn read_u3(&mut self) -> Result<u3, CodeEofError> {
        Ok(self.extract(u3::with(3))?.try_into().expect("bit extractor failure"))
    }

    fn read_u4(&mut self) -> Result<u4, CodeEofError> {
        Ok(self.extract(u3::with(4))?.try_into().expect("bit extractor failure"))
    }

    fn read_u5(&mut self) -> Result<u5, CodeEofError> {
        Ok(self.extract(u3::with(5))?.try_into().expect("bit extractor failure"))
    }

    fn read_u6(&mut self) -> Result<u6, CodeEofError> {
        Ok(self.extract(u3::with(6))?.try_into().expect("bit extractor failure"))
    }

    fn read_u7(&mut self) -> Result<u7, CodeEofError> {
        Ok(self.extract(u3::with(7))?.try_into().expect("bit extractor failure"))
    }

    fn read_u8(&mut self) -> Result<u8, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let byte = self.as_ref()[self.byte_pos as usize];
        self.inc_bytes(1).map(|_| byte)
    }

    fn read_i8(&mut self) -> Result<i8, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let byte = self.as_ref()[self.byte_pos as usize] as i8;
        self.inc_bytes(1).map(|_| byte)
    }

    fn read_u16(&mut self) -> Result<u16, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let pos = self.byte_pos as usize;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&self.as_ref()[pos..pos + 2]);
        let word = u16::from_le_bytes(buf);
        self.inc_bytes(2).map(|_| word)
    }

    fn read_i16(&mut self) -> Result<i16, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let pos = self.byte_pos as usize;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&self.as_ref()[pos..pos + 2]);
        let word = i16::from_le_bytes(buf);
        self.inc_bytes(2).map(|_| word)
    }

    fn read_u24(&mut self) -> Result<u24, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        let pos = self.byte_pos as usize;
        let mut buf = [0u8; 3];
        buf.copy_from_slice(&self.as_ref()[pos..pos + 3]);
        let word = u24::from_le_bytes(buf);
        self.inc_bytes(3).map(|_| word)
    }

    #[inline]
    fn read_lib(&mut self) -> Result<LibId, CodeEofError> {
        Ok(self.libs.at(self.read_u8()?).unwrap_or_default())
    }

    fn read_data(&mut self) -> Result<(&[u8], bool), CodeEofError> {
        let offset = self.read_u16()? as usize;
        let end = offset + self.read_u16()? as usize;
        let max = DATA_SEGMENT_MAX_LEN;
        let st0 = end > self.data.as_ref().len();
        let data = &self.data.as_ref()[offset.min(max)..end.min(max)];
        Ok((data, st0))
    }

    fn read_number(&mut self, reg: impl NumericRegister) -> Result<Number, CodeEofError> {
        let offset = self.read_u16()? as usize;
        let end = offset + reg.bytes() as usize;
        if end > self.data.as_ref().len() {
            return Err(CodeEofError);
        }
        Ok(Number::with(&self.data.as_ref()[offset..end], reg.layout())
            .expect("read_number is broken"))
    }
}

impl<'a, T, D> Write for Cursor<'a, T, D>
where
    T: AsRef<[u8]> + AsMut<[u8]>,
    D: AsRef<[u8]> + AsMut<[u8]> + Extend<u8>,
    Self: 'a,
{
    fn write_bool(&mut self, data: bool) -> Result<(), WriteError> {
        let data = if data { 1u8 } else { 0u8 } << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(1)).map_err(WriteError::from)
    }

    fn write_u1(&mut self, data: impl Into<u1>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(1)).map_err(WriteError::from)
    }

    fn write_u2(&mut self, data: impl Into<u2>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(2)).map_err(WriteError::from)
    }

    fn write_u3(&mut self, data: impl Into<u3>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(3)).map_err(WriteError::from)
    }

    fn write_u4(&mut self, data: impl Into<u4>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(4)).map_err(WriteError::from)
    }

    fn write_u5(&mut self, data: impl Into<u5>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(5)).map_err(WriteError::from)
    }

    fn write_u6(&mut self, data: impl Into<u6>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(6)).map_err(WriteError::from)
    }

    fn write_u7(&mut self, data: impl Into<u7>) -> Result<(), WriteError> {
        let data = data.into().as_u8() << self.bit_pos.as_u8();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] |= data;
        self.inc_bits(u3::with(7)).map_err(WriteError::from)
    }

    fn write_u8(&mut self, data: impl Into<u8>) -> Result<(), WriteError> {
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] = data.into();
        self.inc_bytes(1).map_err(WriteError::from)
    }

    fn write_i8(&mut self, data: impl Into<i8>) -> Result<(), WriteError> {
        let data = data.into().to_le_bytes();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] = data[0];
        self.inc_bytes(1).map_err(WriteError::from)
    }

    fn write_u16(&mut self, data: impl Into<u16>) -> Result<(), WriteError> {
        let data = data.into().to_le_bytes();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] = data[0];
        self.as_mut()[pos + 1] = data[1];
        self.inc_bytes(2).map_err(WriteError::from)
    }

    fn write_i16(&mut self, data: impl Into<i16>) -> Result<(), WriteError> {
        let data = data.into().to_le_bytes();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] = data[0];
        self.as_mut()[pos + 1] = data[1];
        self.inc_bytes(2).map_err(WriteError::from)
    }

    fn write_u24(&mut self, data: impl Into<u24>) -> Result<(), WriteError> {
        let data = data.into().to_le_bytes();
        let pos = self.byte_pos as usize;
        self.as_mut()[pos] = data[0];
        self.as_mut()[pos + 1] = data[1];
        self.as_mut()[pos + 2] = data[2];
        self.inc_bytes(3).map_err(WriteError::from)
    }

    #[inline]
    fn write_lib(&mut self, lib: LibId) -> Result<(), WriteError> {
        self.write_u8(self.libs.index(lib).ok_or(WriteError::LibAbsent(lib))?)
    }

    fn write_data(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), WriteError> {
        // We control that `self.byte_pos + bytes.len() < u16` at buffer
        // allocation time, so if we panic here this means we have a bug in
        // out allocation code and has to kill the process and report this issue
        let bytes = bytes.as_ref();
        let len = bytes.len();
        if len >= u16::MAX as usize {
            return Err(WriteError::DataExceedsLimit(len));
        }
        let offset = self.write_unique(bytes)?;
        self.write_u16(offset)?;
        self.write_u16(len as u16)
    }

    fn write_number(
        &mut self,
        reg: impl NumericRegister,
        mut value: Number,
    ) -> Result<(), WriteError> {
        let len = reg.bytes();
        assert!(
            len <= value.len(),
            "value for the register has larger bit length than the register"
        );
        value.reshape(reg.layout().using_sign(value.layout()));
        let offset = self.write_unique(&value[..])?;
        self.write_u16(offset)
    }

    fn edit<F, E, S>(&mut self, pos: u16, editor: F) -> Result<(), E>
    where
        F: FnOnce(&mut Instr<S>) -> Result<(), E>,
        E: From<CodeEofError>,
        S: InstructionSet,
    {
        let prev_pos = self.seek(pos)?;
        let mut instr = Instr::read(self)?;
        editor(&mut instr)?;
        self.seek(pos)?;
        instr.write(self).expect("cursor editor fail");
        self.seek(prev_pos)?;
        Ok(())
    }
}
