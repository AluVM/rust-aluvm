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

use core::convert::TryInto;
#[cfg(feature = "std")]
use core::fmt::{self, Debug, Display, Formatter};

use amplify::num::{u1, u2, u24, u3, u4, u5, u6, u7};

use super::{CodeEofError, LibId, LibSeg, Read, Write, WriteError};
use crate::data::Number;
use crate::isa::{Bytecode, Instr, InstructionSet};
use crate::library::constants::{CODE_SEGMENT_MAX_LEN, DATA_SEGMENT_MAX_LEN};
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
            .field("program", &self.libs)
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
    /// Creates new cursor able to write the bytecode and data, using provided immutable program
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
    /// Creates cursor from the provided byte string utilizing existing program segment
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

    fn read(&mut self, bit_count: u5) -> Result<u32, CodeEofError> {
        let mut ret = 0u32;
        let mut cnt = bit_count.to_u8();
        while cnt > 0 {
            if self.is_eof() {
                return Err(CodeEofError);
            }
            let byte = self.as_ref()[self.byte_pos as usize];
            let remaining_bits = 8 - self.bit_pos.to_u8();
            let mask = match remaining_bits < cnt {
                true => 0xFFu8 << self.bit_pos.to_u8(),
                false => (((1u16 << (cnt)) - 1) << (self.bit_pos.to_u8() as u16)) as u8,
            };
            let value = ((byte & mask) >> self.bit_pos.to_u8()) as u32;
            ret |= value << (bit_count.to_u8() - cnt);
            match remaining_bits.min(cnt) {
                8 => {
                    self.inc_bytes(1)?;
                }
                _ => {
                    self.inc_bits(u3::with(remaining_bits.min(cnt)))?;
                }
            }
            cnt = cnt.saturating_sub(remaining_bits);
        }
        Ok(ret)
    }

    fn inc_bits(&mut self, bit_count: u3) -> Result<(), CodeEofError> {
        let pos = self.bit_pos.to_u8() + bit_count.to_u8();
        self.bit_pos = u3::with(pos % 8);
        self._inc_bytes_inner(pos as u16 / 8)
    }

    fn inc_bytes(&mut self, byte_count: u16) -> Result<(), CodeEofError> {
        assert_eq!(
            self.bit_pos.to_u8(),
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

    fn write(&mut self, value: u32, bit_count: u5) -> Result<(), CodeEofError> {
        let mut cnt = bit_count.to_u8();
        let value = ((value as u64) << (self.bit_pos.to_u8())).to_le_bytes();
        let n_bytes = (cnt + self.bit_pos.to_u8() + 7) / 8;
        for i in 0..n_bytes {
            if self.is_eof() {
                return Err(CodeEofError);
            }
            let byte_pos = self.byte_pos as usize;
            let bit_pos = self.bit_pos.to_u8();
            let byte = &mut self.as_mut()[byte_pos];
            *byte |= value[i as usize];
            match (bit_pos, cnt) {
                (0, cnt) if cnt >= 8 => {
                    self.inc_bytes(1)?;
                }
                (_, cnt) => {
                    self.inc_bits(u3::with(cnt.min(8 - bit_pos)))?;
                }
            }
            cnt = cnt.saturating_sub(cnt.min(8 - bit_pos));
        }
        Ok(())
    }
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
        if len == 0 {
            Ok(offset as u16)
        } else if let Some(offset) =
            self.data.as_ref().windows(len).position(|window| window == bytes)
        {
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

    fn read_bool(&mut self) -> Result<bool, CodeEofError> { Ok(self.read(u5::with(1))? == 0x01) }

    fn read_u1(&mut self) -> Result<u1, CodeEofError> {
        let res = self.read(u5::with(1))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u2(&mut self) -> Result<u2, CodeEofError> {
        let res = self.read(u5::with(2))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u3(&mut self) -> Result<u3, CodeEofError> {
        let res = self.read(u5::with(3))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u4(&mut self) -> Result<u4, CodeEofError> {
        let res = self.read(u5::with(4))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u5(&mut self) -> Result<u5, CodeEofError> {
        let res = self.read(u5::with(5))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u6(&mut self) -> Result<u6, CodeEofError> {
        let res = self.read(u5::with(6))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u7(&mut self) -> Result<u7, CodeEofError> {
        let res = self.read(u5::with(7))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_u8(&mut self) -> Result<u8, CodeEofError> {
        let res = self.read(u5::with(8))? as u8;
        Ok(res)
    }

    fn read_i8(&mut self) -> Result<i8, CodeEofError> {
        let res = self.read(u5::with(8))? as i8;
        Ok(res)
    }

    fn read_u16(&mut self) -> Result<u16, CodeEofError> {
        let res = self.read(u5::with(16))? as u16;
        Ok(res)
    }

    fn read_i16(&mut self) -> Result<i16, CodeEofError> {
        let res = self.read(u5::with(16))? as i16;
        Ok(res)
    }

    fn read_u24(&mut self) -> Result<u24, CodeEofError> {
        let res = self.read(u5::with(24))?;
        Ok(res.try_into().expect("bit extractor failure"))
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
        self.write(data as u32, u5::with(1)).map_err(WriteError::from)
    }

    fn write_u1(&mut self, data: impl Into<u1>) -> Result<(), WriteError> {
        self.write(data.into().into_u8() as u32, u5::with(1)).map_err(WriteError::from)
    }

    fn write_u2(&mut self, data: impl Into<u2>) -> Result<(), WriteError> {
        self.write(data.into().to_u8() as u32, u5::with(2)).map_err(WriteError::from)
    }

    fn write_u3(&mut self, data: impl Into<u3>) -> Result<(), WriteError> {
        self.write(data.into().to_u8() as u32, u5::with(3)).map_err(WriteError::from)
    }

    fn write_u4(&mut self, data: impl Into<u4>) -> Result<(), WriteError> {
        self.write(data.into().to_u8() as u32, u5::with(4)).map_err(WriteError::from)
    }

    fn write_u5(&mut self, data: impl Into<u5>) -> Result<(), WriteError> {
        self.write(data.into().to_u8() as u32, u5::with(5)).map_err(WriteError::from)
    }

    fn write_u6(&mut self, data: impl Into<u6>) -> Result<(), WriteError> {
        self.write(data.into().to_u8() as u32, u5::with(6)).map_err(WriteError::from)
    }

    fn write_u7(&mut self, data: impl Into<u7>) -> Result<(), WriteError> {
        self.write(data.into().to_u8() as u32, u5::with(7)).map_err(WriteError::from)
    }

    fn write_u8(&mut self, data: impl Into<u8>) -> Result<(), WriteError> {
        self.write(data.into() as u32, u5::with(8)).map_err(WriteError::from)
    }

    fn write_i8(&mut self, data: impl Into<i8>) -> Result<(), WriteError> {
        self.write(data.into() as u32, u5::with(8)).map_err(WriteError::from)
    }

    fn write_u16(&mut self, data: impl Into<u16>) -> Result<(), WriteError> {
        self.write(data.into() as u32, u5::with(16)).map_err(WriteError::from)
    }

    fn write_i16(&mut self, data: impl Into<i16>) -> Result<(), WriteError> {
        self.write(data.into() as u32, u5::with(16)).map_err(WriteError::from)
    }

    fn write_u24(&mut self, data: impl Into<u24>) -> Result<(), WriteError> {
        self.write(data.into().into_u32(), u5::with(24)).map_err(WriteError::from)
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
        let mut instr = Instr::decode(self)?;
        editor(&mut instr)?;
        self.seek(pos)?;
        instr.encode(self).expect("cursor editor fail");
        self.seek(prev_pos)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use amplify::num::{u2, u3, u5, u7};

    use super::Cursor;
    use crate::data::ByteStr;
    use crate::library::{LibSeg, Read, Write};

    #[test]
    fn read() {
        let libseg = LibSeg::default();
        let mut cursor = Cursor::<_, ByteStr>::new([0b01010111, 0b00001001], &libseg);
        assert_eq!(cursor.read_u2().unwrap().to_u8(), 0b00000011);
        assert_eq!(cursor.read_u2().unwrap().to_u8(), 0b00000001);
        assert_eq!(cursor.read_u8().unwrap(), 0b10010101);

        let mut cursor = Cursor::<_, ByteStr>::new([0b01010111, 0b00001001], &libseg);
        assert_eq!(cursor.read_u2().unwrap().to_u8(), 0b00000011);
        assert_eq!(cursor.read_u3().unwrap().to_u8(), 0b00000101);
        assert_eq!(cursor.read_u8().unwrap(), 0b01001010);

        let mut cursor = Cursor::<_, ByteStr>::new([0b01110111, 0b00001111], &libseg);
        assert_eq!(cursor.read_u8().unwrap(), 0b01110111);
        assert_eq!(cursor.read_u3().unwrap().to_u8(), 0b00000111);
        assert_eq!(cursor.read_u5().unwrap().to_u8(), 0b00000001);

        let bytes = 0b11101011_11110000_01110111;
        let mut cursor = Cursor::<_, ByteStr>::new(u32::to_le_bytes(bytes), &libseg);
        assert_eq!(cursor.read(u5::with(24)).unwrap(), bytes);
    }

    #[test]
    fn read_eof() {
        let libseg = LibSeg::default();
        let mut cursor = Cursor::<_, ByteStr>::new([0b01010111], &libseg);
        assert_eq!(cursor.read_u2().unwrap().to_u8(), 0b00000011);
        assert_eq!(cursor.read_u2().unwrap().to_u8(), 0b00000001);
        assert!(cursor.read_u8().is_err());
    }

    #[test]
    fn write() {
        let libseg = LibSeg::default();
        let mut code = [0, 0, 0, 0, 0, 0];
        let mut cursor = Cursor::<_, ByteStr>::new(&mut code, &libseg);
        cursor.write_u2(u2::with(0b00000011)).unwrap();
        cursor.write_u3(u3::with(0b00000101)).unwrap();
        cursor.write_u7(u7::with(0b01011111)).unwrap();
        cursor.write_u8(0b11100111).unwrap();
        cursor.write_bool(true).unwrap();
        cursor.write_u3(u3::with(0b00000110)).unwrap();
        let two_bytes = 0b11110000_10101010u16;
        cursor.write_u16(two_bytes).unwrap();

        let mut cursor = Cursor::<_, ByteStr>::new(code, &libseg);
        assert_eq!(cursor.read_u2().unwrap().to_u8(), 0b00000011);
        assert_eq!(cursor.read_u3().unwrap().to_u8(), 0b00000101);
        assert_eq!(cursor.read_u7().unwrap().to_u8(), 0b01011111);
        assert_eq!(cursor.read_u8().unwrap(), 0b11100111);
        assert_eq!(cursor.read_bool().unwrap(), true);
        assert_eq!(cursor.read_u3().unwrap().to_u8(), 0b00000110);
        assert_eq!(cursor.read_u16().unwrap(), two_bytes);
    }

    #[test]
    fn write_eof() {
        let libseg = LibSeg::default();
        let mut code = [0, 0];
        let mut cursor = Cursor::<_, ByteStr>::new(&mut code, &libseg);
        cursor.write_u2(u2::with(0b00000011)).unwrap();
        cursor.write_u3(u3::with(0b00000101)).unwrap();
        cursor.write_u7(u7::with(0b01011111)).unwrap();
        assert!(cursor.write_u8(0b11100111).is_err());
    }
}
