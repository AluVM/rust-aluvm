// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 UBIDECO Labs,
//     Institute for Distributed and Cognitive Computing, Switzerland.
//     All rights reserved.
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

use core::fmt::{self, Debug, Formatter};

use amplify::confinement::SmallBlob;
use amplify::num::{u1, u2, u3, u4, u5, u6, u7};

use super::{LibId, LibsSeg};
use crate::isa::{BytecodeRead, BytecodeWrite, CodeEofError};

/// Errors write operations
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum MarshallError {
    /// attempt to read or write outside of code segment (i.e. at position > 0xFF).
    #[from(CodeEofError)]
    CodeNotFittingSegment,

    /// data size {0} exceeds limit of 0xFF bytes.
    DataExceedsLimit(usize),

    /// attempt to write data which does not fit code segment.
    DataNotFittingSegment,

    /// attempt to write library reference for the lib id {0} which is not a part of program
    /// segment.
    LibAbsent(LibId),
}

/// Marshals instructions to and from bytecode representation.
pub struct Marshaller<'a, C, D>
where
    C: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    bit_pos: u3,
    byte_pos: u16,
    bytecode: C,
    data: D,
    libs: &'a LibsSeg,
}

impl<'a, C, D> Debug for Marshaller<'a, C, D>
where
    C: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Marshaller")
            .field("bytecode", &SmallBlob::from_slice_checked(self.bytecode.as_ref()))
            .field("byte_pos", &self.byte_pos)
            .field("bit_pos", &self.bit_pos)
            .field("data", &SmallBlob::from_slice_checked(self.data.as_ref()))
            .field("libs", &self.libs)
            .finish()
    }
}

impl<'a> Marshaller<'a, Vec<u8>, Vec<u8>>
where Self: 'a
{
    /// Creates a new marshaller using provided set of libraries.
    #[inline]
    pub fn new(libs: &'a LibsSeg) -> Self {
        Self {
            bytecode: default!(),
            byte_pos: 0,
            bit_pos: u3::MIN,
            data: default!(),
            libs,
        }
    }

    /// Completes marshalling, returning produced data segment.
    ///
    /// # Panics
    ///
    /// If marshaller position is not at byte margin.
    #[inline]
    pub fn finish(self) -> (SmallBlob, SmallBlob) {
        if self.bit_pos != u3::ZERO {
            panic!("incomplete marshalling")
        }
        (SmallBlob::from_checked(self.bytecode), SmallBlob::from_checked(self.data))
    }
}

impl<'a, C, D> Marshaller<'a, C, D>
where
    C: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    /// Create marshaller from byte string utilizing existing bytecode.
    ///
    /// # Panics
    ///
    /// If the length of the bytecode or data segment exceeds 0xFF.
    #[inline]
    pub fn with(bytecode: C, data: D, libs: &'a LibsSeg) -> Self {
        Self {
            bytecode,
            byte_pos: 0,
            bit_pos: u3::MIN,
            data,
            libs,
        }
    }

    /// Returns the current offset of the marshaller
    pub const fn offset(&self) -> (u16, u3) { (self.byte_pos, self.bit_pos) }

    fn read(&mut self, bit_count: u5) -> Result<u32, CodeEofError> {
        let mut ret = 0u32;
        let mut cnt = bit_count.to_u8();
        while cnt > 0 {
            if self.is_eof() {
                return Err(CodeEofError);
            }
            let byte = &self.bytecode.as_ref()[self.byte_pos as usize];
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
        assert_eq!(self.bit_pos.to_u8(), 0, "attempt to access (multiple) bytes at a non-byte aligned position");
        self._inc_bytes_inner(byte_count)
    }

    #[inline]
    fn _inc_bytes_inner(&mut self, byte_count: u16) -> Result<(), CodeEofError> {
        self.byte_pos = self.byte_pos.checked_add(byte_count).ok_or(CodeEofError)?;
        Ok(())
    }
}

impl<'a, C, D> Marshaller<'a, C, D>
where
    C: AsRef<[u8]> + AsMut<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
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
            let byte = &mut self.bytecode.as_mut()[byte_pos];
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

impl<'a, C, D> Marshaller<'a, C, D>
where
    C: AsRef<[u8]> + AsMut<[u8]>,
    D: AsRef<[u8]> + AsMut<[u8]> + Extend<u8>,
    Self: 'a,
{
    fn write_unique(&mut self, bytes: &[u8]) -> Result<u16, MarshallError> {
        // We write the value only if the value is not yet present in the data segment
        let len = bytes.len();
        let offset = self.data.as_ref().len();
        if len == 0 {
            Ok(offset as u16)
        } else if let Some(offset) = self
            .data
            .as_ref()
            .windows(len)
            .position(|window| window == bytes)
        {
            Ok(offset as u16)
        } else if offset + len > u16::MAX as usize {
            Err(MarshallError::DataNotFittingSegment)
        } else {
            self.data.extend(bytes.iter().copied());
            Ok(offset as u16)
        }
    }
}

impl<'a, C, D> BytecodeRead<LibId> for Marshaller<'a, C, D>
where
    C: AsRef<[u8]>,
    D: AsRef<[u8]>,
    Self: 'a,
{
    #[inline]
    fn pos(&self) -> u16 { self.byte_pos }

    #[inline]
    fn seek(&mut self, byte_pos: u16) -> Result<u16, CodeEofError> {
        if byte_pos as usize >= self.bytecode.as_ref().len() {
            return Err(CodeEofError);
        }
        let old_pos = self.byte_pos;
        self.byte_pos = byte_pos;
        Ok(old_pos)
    }

    #[inline]
    fn is_eof(&self) -> bool { self.byte_pos as usize >= self.bytecode.as_ref().len() }

    fn peek_byte(&self) -> Result<u8, CodeEofError> {
        if self.is_eof() {
            return Err(CodeEofError);
        }
        Ok(self.bytecode.as_ref()[self.byte_pos as usize])
    }

    fn read_bool(&mut self) -> Result<bool, CodeEofError> { Ok(self.read(u5::with(1))? == 0x01) }

    fn read_1bit(&mut self) -> Result<u1, CodeEofError> {
        let res = self.read(u5::with(1))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_2bits(&mut self) -> Result<u2, CodeEofError> {
        let res = self.read(u5::with(2))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_3bits(&mut self) -> Result<u3, CodeEofError> {
        let res = self.read(u5::with(3))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_4bits(&mut self) -> Result<u4, CodeEofError> {
        let res = self.read(u5::with(4))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_5bits(&mut self) -> Result<u5, CodeEofError> {
        let res = self.read(u5::with(5))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_6bits(&mut self) -> Result<u6, CodeEofError> {
        let res = self.read(u5::with(6))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_7bits(&mut self) -> Result<u7, CodeEofError> {
        let res = self.read(u5::with(7))? as u8;
        Ok(res.try_into().expect("bit extractor failure"))
    }

    fn read_byte(&mut self) -> Result<u8, CodeEofError> {
        let res = self.read(u5::with(8))? as u8;
        Ok(res)
    }

    fn read_word(&mut self) -> Result<u16, CodeEofError> {
        let res = self.read(u5::with(16))? as u16;
        Ok(res)
    }

    fn read_fixed<N, const LEN: usize>(&mut self, f: impl FnOnce([u8; LEN]) -> N) -> Result<N, CodeEofError> {
        let pos = self.read_word()? as usize;
        let end = pos + LEN;
        if end > self.data.as_ref().len() {
            return Err(CodeEofError);
        }
        let mut buf = [0u8; LEN];
        buf.copy_from_slice(&self.data.as_ref()[pos..end]);
        Ok(f(buf))
    }

    fn read_bytes(&mut self) -> Result<(SmallBlob, bool), CodeEofError> {
        let pos = self.read_word()? as usize;
        let end = pos + self.read_word()? as usize;
        let ck = end >= self.data.as_ref().len();
        let data = &self.data.as_ref()[pos.min(0xFF)..end.min(0xFF)];
        Ok((SmallBlob::from_slice_checked(data), ck))
    }

    fn read_ref(&mut self) -> Result<LibId, CodeEofError>
    where LibId: Sized {
        let pos = self.read_byte()? as usize;
        Ok(self.libs.iter().nth(pos).copied().unwrap_or_default())
    }

    fn check_aligned(&self) { debug_assert_eq!(self.bit_pos, u3::ZERO, "not all instruction operands are read") }
}

impl<'a, C, D> BytecodeWrite<LibId> for Marshaller<'a, C, D>
where
    C: AsRef<[u8]> + AsMut<[u8]>,
    D: AsRef<[u8]> + AsMut<[u8]> + Extend<u8>,
    Self: 'a,
{
    type Error = MarshallError;

    fn write_1bit(&mut self, data: impl Into<u1>) -> Result<(), MarshallError> {
        self.write(data.into().into_u8() as u32, u5::with(1))
            .map_err(MarshallError::from)
    }

    fn write_2bits(&mut self, data: impl Into<u2>) -> Result<(), MarshallError> {
        self.write(data.into().to_u8() as u32, u5::with(2))
            .map_err(MarshallError::from)
    }

    fn write_3bits(&mut self, data: impl Into<u3>) -> Result<(), MarshallError> {
        self.write(data.into().to_u8() as u32, u5::with(3))
            .map_err(MarshallError::from)
    }

    fn write_4bits(&mut self, data: impl Into<u4>) -> Result<(), MarshallError> {
        self.write(data.into().to_u8() as u32, u5::with(4))
            .map_err(MarshallError::from)
    }

    fn write_5bits(&mut self, data: impl Into<u5>) -> Result<(), MarshallError> {
        self.write(data.into().to_u8() as u32, u5::with(5))
            .map_err(MarshallError::from)
    }

    fn write_6bits(&mut self, data: impl Into<u6>) -> Result<(), MarshallError> {
        self.write(data.into().to_u8() as u32, u5::with(6))
            .map_err(MarshallError::from)
    }

    fn write_7bits(&mut self, data: impl Into<u7>) -> Result<(), MarshallError> {
        self.write(data.into().to_u8() as u32, u5::with(7))
            .map_err(MarshallError::from)
    }

    fn write_byte(&mut self, data: u8) -> Result<(), MarshallError> {
        self.write(data as u32, u5::with(8))
            .map_err(MarshallError::from)
    }

    fn write_word(&mut self, data: u16) -> Result<(), MarshallError> {
        self.write(data as u32, u5::with(16))
            .map_err(MarshallError::from)
    }

    fn write_fixed<const LEN: usize>(&mut self, data: [u8; LEN]) -> Result<(), Self::Error> {
        if LEN >= u16::MAX as usize {
            return Err(MarshallError::DataExceedsLimit(LEN));
        }
        let offset = self.write_unique(&data)?;
        self.write_word(offset)
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let len = data.len();
        if len >= u16::MAX as usize {
            return Err(MarshallError::DataExceedsLimit(len));
        }
        let offset = self.write_unique(&data)?;
        self.write_word(offset)?;
        self.write_word(len as u16)
    }

    fn write_ref(&mut self, id: LibId) -> Result<(), Self::Error> {
        let pos = self
            .libs
            .iter()
            .position(|lib| *lib == id)
            .ok_or(MarshallError::LibAbsent(id))?;
        self.write_byte(pos as u8)
    }

    fn check_aligned(&self) { debug_assert_eq!(self.bit_pos, u3::ZERO, "not all instruction operands are written") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        let libseg = LibsSeg::default();
        let mut marshaller = Marshaller::with([0b01010111, 0b00001001], [], &libseg);
        assert_eq!(marshaller.read_2bits().unwrap().to_u8(), 0b00000011);
        assert_eq!(marshaller.read_2bits().unwrap().to_u8(), 0b00000001);
        assert_eq!(marshaller.read_byte().unwrap(), 0b10010101);

        let mut marshaller = Marshaller::with([0b01010111, 0b00001001], [], &libseg);
        assert_eq!(marshaller.read_2bits().unwrap().to_u8(), 0b00000011);
        assert_eq!(marshaller.read_3bits().unwrap().to_u8(), 0b00000101);
        assert_eq!(marshaller.read_byte().unwrap(), 0b01001010);

        let mut marshaller = Marshaller::with([0b01110111, 0b00001111], [], &libseg);
        assert_eq!(marshaller.read_byte().unwrap(), 0b01110111);
        assert_eq!(marshaller.read_3bits().unwrap().to_u8(), 0b00000111);
        assert_eq!(marshaller.read_5bits().unwrap().to_u8(), 0b00000001);

        let bytes = 0b11101011_11110000_01110111;
        let mut marshaller = Marshaller::with(u32::to_le_bytes(bytes), [], &libseg);
        assert_eq!(marshaller.read(u5::with(24)).unwrap(), bytes);
    }

    #[test]
    fn read_eof() {
        let libseg = LibsSeg::default();
        let mut marshaller = Marshaller::with([0b01010111], [], &libseg);
        assert_eq!(marshaller.read_2bits().unwrap().to_u8(), 0b00000011);
        assert_eq!(marshaller.read_2bits().unwrap().to_u8(), 0b00000001);
        assert!(marshaller.read_byte().is_err());
    }

    #[test]
    fn write() {
        let libseg = LibsSeg::default();
        let mut code = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut marshaller = Marshaller::with(&mut code, vec![], &libseg);
        marshaller.write_2bits(u2::with(0b00000011)).unwrap();
        marshaller.write_3bits(u3::with(0b00000101)).unwrap();
        marshaller.write_7bits(u7::with(0b01011111)).unwrap();
        marshaller.write_byte(0b11100111).unwrap();
        marshaller.write_bool(true).unwrap();
        marshaller.write_3bits(u3::with(0b00000110)).unwrap();
        let two_bytes = 0b11110000_10101010u16;
        marshaller.write_word(two_bytes).unwrap();
        let number = 255u8;
        marshaller.write_fixed(255u8.to_le_bytes()).unwrap();

        let data = marshaller.data;
        let mut marshaller = Marshaller::with(code, data, &libseg);
        assert_eq!(marshaller.read_2bits().unwrap().to_u8(), 0b00000011);
        assert_eq!(marshaller.read_3bits().unwrap().to_u8(), 0b00000101);
        assert_eq!(marshaller.read_7bits().unwrap().to_u8(), 0b01011111);
        assert_eq!(marshaller.read_byte().unwrap(), 0b11100111);
        assert!(marshaller.read_bool().unwrap());
        assert_eq!(marshaller.read_3bits().unwrap().to_u8(), 0b00000110);
        assert_eq!(marshaller.read_word().unwrap(), two_bytes);
        assert_eq!(marshaller.read_fixed(u8::from_le_bytes).unwrap(), number);
    }

    #[test]
    fn write_data() {
        let libseg = LibsSeg::default();
        let mut code = [0, 0, 0, 0, 0, 0];
        let mut marshaller = Marshaller::with(&mut code, vec![], &libseg);
        marshaller.write_fixed(256u16.to_le_bytes()).unwrap();
        assert_eq!(marshaller.data, vec![0, 1]);
    }

    #[test]
    fn write_eof() {
        let libseg = LibsSeg::default();
        let mut code = [0, 0];
        let mut marshaller = Marshaller::with(&mut code, vec![], &libseg);
        marshaller.write_2bits(u2::with(0b00000011)).unwrap();
        marshaller.write_3bits(u3::with(0b00000101)).unwrap();
        marshaller.write_7bits(u7::with(0b01011111)).unwrap();
        assert!(marshaller.write_byte(0b11100111).is_err());
    }
}
