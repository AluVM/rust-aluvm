// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Labs,
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

use core::ops::RangeInclusive;

use amplify::num::{u1, u2, u24, u3, u4, u5, u6, u7};

use super::{LibId, LibSite};
use crate::data::Number;
use crate::reg::NumericRegister;

// I had an idea of putting Read/Write functionality into `amplify` crate,
// but it is quire specific to the fact that it uses `u16`-sized underlying
// bytestring, which is specific to client-side-validation and this VM and not
// generic enough to become part of the `amplify` library

/// Error indicating that an end of code segment boundary is reached during read or write operation
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display("attempt to read or write outside of code segment (i.e. at position > 2^16)")]
#[cfg_attr(feature = "std", derive(Error))]
pub struct CodeEofError;

/// Errors write operations
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum WriteError {
    /// attempt to read or write outside of code segment (i.e. at position > 2^16)
    #[from(CodeEofError)]
    CodeNotFittingSegment,

    /// data size {0} exceeds limit of 2^16 bytes
    DataExceedsLimit(usize),

    /// attempt to write data which does not fit code segment
    DataNotFittingSegment,

    /// attempt to write library reference for the lib id {0} which is not a part of program
    /// segment
    LibAbsent(LibId),
}

mod private {
    use super::super::Cursor;

    pub trait Sealed {}

    impl<'a, T, D> Sealed for Cursor<'a, T, D>
    where
        T: AsRef<[u8]>,
        D: AsRef<[u8]>,
        Self: 'a,
    {
    }
}

/// Trait for reading instruction data from bytecode
pub trait Read: private::Sealed {
    /// Returns current byte offset of the cursor. Does not accounts bits.
    /// If the position is exactly at EOF, returns `None`.
    fn pos(&self) -> u16;
    /// Sets current cursor byte offset to the provided value, if it is less than the underlying
    /// buffer length
    ///
    /// # Returns
    ///
    /// Previous position
    fn seek(&mut self, byte_pos: u16) -> Result<u16, CodeEofError>;
    /// Returns whether end of the bytecode is reached
    fn is_eof(&self) -> bool;
    /// Peeks a single byte without moving cursor
    fn peek_u8(&self) -> Result<u8, CodeEofError>;
    /// Reads single bit as a bool values
    fn read_bool(&mut self) -> Result<bool, CodeEofError>;
    /// Reads single bit
    fn read_u1(&mut self) -> Result<u1, CodeEofError>;
    /// Reads two bits
    fn read_u2(&mut self) -> Result<u2, CodeEofError>;
    /// Reads three bits
    fn read_u3(&mut self) -> Result<u3, CodeEofError>;
    /// Reads four bits
    fn read_u4(&mut self) -> Result<u4, CodeEofError>;
    /// Reads five bits
    fn read_u5(&mut self) -> Result<u5, CodeEofError>;
    /// Reads six bits
    fn read_u6(&mut self) -> Result<u6, CodeEofError>;
    /// Reads seven bits
    fn read_u7(&mut self) -> Result<u7, CodeEofError>;
    /// Reads full byte
    fn read_u8(&mut self) -> Result<u8, CodeEofError>;
    /// Reads two bytes and converts them into a signed integer
    fn read_i8(&mut self) -> Result<i8, CodeEofError>;
    /// Reads two bytes
    fn read_u16(&mut self) -> Result<u16, CodeEofError>;
    /// Reads two bytes and converts them into a signed integer
    fn read_i16(&mut self) -> Result<i16, CodeEofError>;
    /// Reads three bytes
    fn read_u24(&mut self) -> Result<u24, CodeEofError>;
    /// Reads library id
    fn read_lib(&mut self) -> Result<LibId, CodeEofError>;
    /// Reads bytestring from data segment
    fn read_data(&mut self) -> Result<(&[u8], bool), CodeEofError>;
    /// Reads number representation from a data segment
    fn read_number(&mut self, reg: impl NumericRegister) -> Result<Number, CodeEofError>;
}

/// Trait for writing instruction data into bytecode
pub trait Write: private::Sealed {
    /// Writes a single bit from a bool value
    fn write_bool(&mut self, data: bool) -> Result<(), WriteError>;
    /// Writes a single bit
    fn write_u1(&mut self, data: impl Into<u1>) -> Result<(), WriteError>;
    /// Writes two bits
    fn write_u2(&mut self, data: impl Into<u2>) -> Result<(), WriteError>;
    /// Writes three bits
    fn write_u3(&mut self, data: impl Into<u3>) -> Result<(), WriteError>;
    /// Writes four bits
    fn write_u4(&mut self, data: impl Into<u4>) -> Result<(), WriteError>;
    /// Writes five bits
    fn write_u5(&mut self, data: impl Into<u5>) -> Result<(), WriteError>;
    /// Writes six bits
    fn write_u6(&mut self, data: impl Into<u6>) -> Result<(), WriteError>;
    /// Writes seven bits
    fn write_u7(&mut self, data: impl Into<u7>) -> Result<(), WriteError>;
    /// Writes full byte
    fn write_u8(&mut self, data: impl Into<u8>) -> Result<(), WriteError>;
    /// Writes full byte corresponding to signed integer representation
    fn write_i8(&mut self, data: impl Into<i8>) -> Result<(), WriteError>;
    /// Writes two bytes
    fn write_u16(&mut self, data: impl Into<u16>) -> Result<(), WriteError>;
    /// Writes two bytes corresponding to signed integer representation
    fn write_i16(&mut self, data: impl Into<i16>) -> Result<(), WriteError>;
    /// Writes three bytes
    fn write_u24(&mut self, data: impl Into<u24>) -> Result<(), WriteError>;
    /// Writes library id into data segment
    fn write_lib(&mut self, data: LibId) -> Result<(), WriteError>;
    /// Writes bytestring into data segment
    fn write_data(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), WriteError>;
    /// Writes number representation into data segment
    fn write_number(&mut self, reg: impl NumericRegister, value: Number) -> Result<(), WriteError>;
}

/// Errors encoding instructions
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(doc_comments)]
pub enum BytecodeError {
    /// Write error
    #[display(inner)]
    #[from]
    Write(WriteError),

    /// put operation does not contain number (when it was deserialized, the data segment was
    /// shorter than the number value offset to read)
    PutNoNumber,
}

#[cfg(feature = "std")]
impl ::std::error::Error for BytecodeError {
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            BytecodeError::Write(err) => Some(err),
            BytecodeError::PutNoNumber => None,
        }
    }
}

/// Non-failiable byte encoding for the instruction set. We can't use `io` since
/// (1) we are no_std, (2) it operates data with unlimited length (while we are
/// bound by u16), (3) it provides too many fails in situations when we can't
/// fail because of `u16`-bounding and exclusive in-memory encoding handling.
pub trait Bytecode {
    /// Returns range of instruction btecodes covered by a set of operations
    fn instr_range() -> RangeInclusive<u8>;

    /// Returns byte representing instruction code (without its arguments)
    fn instr_byte(&self) -> u8;

    /// If the instruction call or references any external library, returns the call site in that
    /// library.
    ///
    /// This is used by jump and subroutine call instructions.
    #[inline]
    fn call_site(&self) -> Option<LibSite> { None }

    /// Writes the instruction as bytecode
    fn encode<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        writer.write_u8(self.instr_byte())?;
        self.encode_args(writer)
    }

    /// Writes instruction arguments as bytecode, omitting instruction code byte
    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write;

    /// Reads the instruction from bytecode
    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read;
}
