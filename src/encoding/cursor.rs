// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::{u2, u3, u4, u5, u6, u7};
use core::convert::TryInto;
#[cfg(feature = "std")]
use std::fmt::{self, Debug, Display, Formatter};

use super::{Read, Write};
use crate::reg::{Reg, Value};

// I had an idea of putting Read/Write functionality into `amplify` crate,
// but it is quire specific to the fact that it uses `u16`-sized underlying
// bytestring, which is specific to client-side-validation and this VM and not
// generic enough to become part of the `amplify` library

/// Errors with cursor-based operations
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display(doc_comments)]
#[derive(Error)]
pub enum CursorError {
    /// Attempt to read or write after end of data
    Eof,

    /// Attempt to read or write at a position outside of data boundaries ({0})
    OutOfBoundaries(usize),
}

/// Cursor for accessing byte string data bounded by `u16::MAX` length
pub struct Cursor<T>
where
    T: AsRef<[u8]>,
{
    bytecode: T,
    byte_pos: u16,
    bit_pos: u3,
    eof: bool,
}

#[cfg(feature = "std")]
impl<T> Debug for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use bitcoin_hashes::hex::ToHex;
        f.debug_struct("Cursor")
            .field("bytecode", &self.bytecode.as_ref().to_hex())
            .field("byte_pos", &self.byte_pos)
            .field("bit_pos", &self.bit_pos)
            .field("eof", &self.eof)
            .finish()
    }
}

#[cfg(feature = "std")]
impl<T> Display for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use bitcoin_hashes::hex::ToHex;
        write!(f, "{}:{} @ ", self.byte_pos, self.bit_pos)?;
        let hex = self.bytecode.as_ref().to_hex();
        if f.alternate() {
            write!(f, "{}..{}", &hex[..4], &hex[hex.len() - 4..])
        } else {
            f.write_str(&hex)
        }
    }
}

impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    /// Creates cursor from the provided byte string
    pub fn with(bytecode: T) -> Cursor<T> {
        Cursor {
            bytecode,
            byte_pos: 0,
            bit_pos: u3::MIN,
            eof: false,
        }
    }

    /// Returns whether cursor is at the upper length boundary for any byte
    /// string (equal to `u16::MAX`)
    pub fn is_eof(&self) -> bool {
        self.eof
    }

    /// Returns current byte offset of the cursor. Does not accounts bits.
    pub fn pos(&self) -> u16 {
        self.byte_pos
    }

    /// Sets current cursor byte offset to the provided value
    pub fn seek(&mut self, byte_pos: u16) {
        self.byte_pos = byte_pos;
    }

    fn extract(&mut self, bit_count: u3) -> Result<u8, CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let byte = self.bytecode.as_ref()[self.byte_pos as usize];
        let mut mask = 0x00u8;
        let mut cnt = *bit_count;
        while cnt > 0 {
            mask <<= 1;
            mask |= 0x01;
            cnt -= 1;
        }
        mask <<= *self.bit_pos;
        let val = (byte & mask) >> *self.bit_pos;
        self.inc_bits(bit_count).map(|_| val)
    }

    fn inc_bits(&mut self, bit_count: u3) -> Result<(), CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let pos = *self.bit_pos + *bit_count;
        self.bit_pos = u3::with(pos % 8);
        self._inc_bytes_inner(pos as u16 / 8)
    }

    fn inc_bytes(&mut self, byte_count: u16) -> Result<(), CursorError> {
        assert_eq!(
            *self.bit_pos, 0,
            "attempt to access (multiple) bytes at a non-byte aligned position"
        );
        if self.eof {
            return Err(CursorError::Eof);
        }
        self._inc_bytes_inner(byte_count)
    }

    fn _inc_bytes_inner(&mut self, byte_count: u16) -> Result<(), CursorError> {
        if byte_count == 1 && self.byte_pos == u16::MAX {
            self.eof = true
        } else {
            self.byte_pos = self.byte_pos.checked_add(byte_count).ok_or(
                CursorError::OutOfBoundaries(
                    self.byte_pos as usize + byte_count as usize,
                ),
            )?;
        }
        Ok(())
    }
}

impl Read for Cursor<&[u8]> {
    type Error = CursorError;

    fn is_end(&self) -> bool {
        self.byte_pos as usize >= self.bytecode.len()
    }

    fn peek_u8(&self) -> Result<u8, CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        Ok(self.bytecode[self.byte_pos as usize])
    }

    fn read_bool(&mut self) -> Result<bool, CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let byte = self.extract(u3::with(1))?;
        Ok(byte == 0x01)
    }

    fn read_u2(&mut self) -> Result<u2, CursorError> {
        Ok(self
            .extract(u3::with(2))?
            .try_into()
            .expect("bit extractor failure"))
    }

    fn read_u3(&mut self) -> Result<u3, CursorError> {
        Ok(self
            .extract(u3::with(3))?
            .try_into()
            .expect("bit extractor failure"))
    }

    fn read_u4(&mut self) -> Result<u4, CursorError> {
        Ok(self
            .extract(u3::with(4))?
            .try_into()
            .expect("bit extractor failure"))
    }

    fn read_u5(&mut self) -> Result<u5, CursorError> {
        Ok(self
            .extract(u3::with(5))?
            .try_into()
            .expect("bit extractor failure"))
    }

    fn read_u6(&mut self) -> Result<u6, CursorError> {
        Ok(self
            .extract(u3::with(6))?
            .try_into()
            .expect("bit extractor failure"))
    }

    fn read_u7(&mut self) -> Result<u7, CursorError> {
        Ok(self
            .extract(u3::with(7))?
            .try_into()
            .expect("bit extractor failure"))
    }

    fn read_u8(&mut self) -> Result<u8, CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let byte = self.bytecode[self.byte_pos as usize];
        self.inc_bytes(1).map(|_| byte)
    }

    fn read_u16(&mut self) -> Result<u16, CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let pos = self.byte_pos as usize;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&self.bytecode[pos..pos + 2]);
        let word = u16::from_le_bytes(buf);
        self.inc_bytes(2).map(|_| word)
    }

    fn read_bytes32(&mut self) -> Result<[u8; 32], CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let pos = self.byte_pos as usize;
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&self.bytecode[pos..pos + 32]);
        self.inc_bytes(32).map(|_| buf)
    }

    fn read_slice(&mut self) -> Result<&[u8], CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let len = self.read_u16()? as usize;
        let pos = self.byte_pos as usize;
        self.inc_bytes(2u16 + len as u16)
            .map(|_| &self.bytecode[pos..pos + len])
    }

    fn read_value(&mut self, reg: Reg) -> Result<Value, CursorError> {
        if self.eof {
            return Err(CursorError::Eof);
        }
        let len = match reg.bits() {
            Some(bits) => bits / 8,
            None => self.read_u16()?,
        } as usize;
        let pos = self.byte_pos as usize;
        let value = Value::with(&self.bytecode[pos..pos + len]);
        self.inc_bytes(len as u16).map(|_| value)
    }
}

impl Write for Cursor<&mut [u8]> {
    type Error = CursorError;

    fn write_bool(&mut self, data: bool) -> Result<(), CursorError> {
        let data = if data { 1u8 } else { 0u8 } << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(1))
    }

    fn write_u2(&mut self, data: impl Into<u2>) -> Result<(), CursorError> {
        let data = data.into().as_u8() << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(2))
    }

    fn write_u3(&mut self, data: impl Into<u3>) -> Result<(), CursorError> {
        let data = data.into().as_u8() << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(3))
    }

    fn write_u4(&mut self, data: impl Into<u4>) -> Result<(), CursorError> {
        let data = data.into().as_u8() << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(4))
    }

    fn write_u5(&mut self, data: impl Into<u5>) -> Result<(), CursorError> {
        let data = data.into().as_u8() << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(5))
    }

    fn write_u6(&mut self, data: impl Into<u6>) -> Result<(), CursorError> {
        let data = data.into().as_u8() << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(6))
    }

    fn write_u7(&mut self, data: impl Into<u7>) -> Result<(), CursorError> {
        let data = data.into().as_u8() << *self.bit_pos;
        self.bytecode[self.byte_pos as usize] |= data;
        self.inc_bits(u3::with(7))
    }

    fn write_u8(&mut self, data: impl Into<u8>) -> Result<(), CursorError> {
        self.bytecode[self.byte_pos as usize] = data.into();
        self.inc_bytes(1)
    }

    fn write_u16(&mut self, data: impl Into<u16>) -> Result<(), CursorError> {
        let data = data.into().to_le_bytes();
        self.bytecode[self.byte_pos as usize] = data[0];
        self.bytecode[self.byte_pos as usize + 1] = data[1];
        self.inc_bytes(2)
    }

    fn write_bytes32(&mut self, data: [u8; 32]) -> Result<(), CursorError> {
        let from = self.byte_pos as usize;
        let to = from + 32;
        self.bytecode[from..to].copy_from_slice(&data);
        self.inc_bytes(32)
    }

    fn write_slice(
        &mut self,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), CursorError> {
        // We control that `self.byte_pos + bytes.len() < u16` at buffer
        // allocation time, so if we panic here this means we have a bug in
        // out allocation code and has to kill the process and report this issue
        let len = bytes.as_ref().len();
        if len >= u16::MAX as usize {
            return Err(CursorError::OutOfBoundaries(len));
        }
        self.write_u16(len as u16)?;
        let from = self.byte_pos as usize;
        let to = from + len;
        self.bytecode[from..to].copy_from_slice(bytes.as_ref());
        self.inc_bytes(2u16 + len as u16)
    }

    fn write_value(
        &mut self,
        reg: Reg,
        value: &Value,
    ) -> Result<(), CursorError> {
        let len = match reg.bits() {
            Some(bits) => bits / 8,
            None => {
                self.write_u16(value.len)?;
                value.len
            }
        };
        assert!(
            len >= value.len,
            "value for the register has larger bit length than the register"
        );
        let value_len = value.len as usize;
        let from = self.byte_pos as usize;
        let to = from + value_len;
        self.bytecode[from..to].copy_from_slice(&value.bytes[0..value_len]);
        self.inc_bytes(len as u16)
    }
}
