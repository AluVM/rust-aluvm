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

use core::error::Error;
use core::fmt::Debug;
use core::ops::RangeInclusive;

use amplify::confinement::SmallBlob;
use amplify::num::{u1, u2, u3, u4, u5, u6, u7};

use crate::core::SiteId;

/// Non-failing byte encoding for the instruction set.
///
/// We can't use `io` since (1) we are no_std, (2) it operates data with unlimited length (while we
/// are bound by u16), (3) it provides too many fails in situations when we can't fail because of
/// `u16`-bounding and exclusive in-memory encoding handling.
pub trait Bytecode<Id: SiteId> {
    /// Returns range of instruction bytecodes covered by a set of operations.
    fn op_range() -> RangeInclusive<u8>;

    /// Returns byte representing instruction code (without its arguments).
    fn opcode_byte(&self) -> u8;

    /// If the instruction call or references any external program, returns a reference to it.
    #[inline]
    fn external_ref(&self) -> Option<Id> { None }

    /// Write an instruction as bytecode.
    fn encode_instr<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        writer.write_byte(self.opcode_byte())?;
        self.encode_operands(writer)?;
        writer.check_aligned();
        Ok(())
    }

    /// Writes an instruction operands as bytecode, omitting opcode byte.
    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id>;

    /// Reads an instruction from bytecode.
    fn decode_instr<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        let opcode = reader.read_byte()?;
        let instr = Self::decode_operands(reader, opcode)?;
        reader.check_aligned();
        Ok(instr)
    }

    /// Reads an instruction operands from bytecode, provided the opcode byte.
    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>;
}

/// Error indicating that an end of code segment boundary is reached during read or write operation.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display("attempt to read or write outside of code segment (i.e. at position > 0xFFFF)")]
pub struct CodeEofError;

/// Reader from a bytecode for instruction deserialization.
pub trait BytecodeRead<Id: SiteId> {
    /// Return current byte offset of the cursor. Does not account for bits.
    /// If the position is exactly at EOF, returns `None`.
    fn pos(&self) -> u16;
    /// Set current cursor byte offset to the provided value, if it is less than the underlying
    /// buffer length.
    ///
    /// # Returns
    ///
    /// Previous position.
    fn seek(&mut self, byte_pos: u16) -> Result<u16, CodeEofError>;
    /// Return whether end of the bytecode is reached.
    fn is_eof(&self) -> bool;
    /// Peek a single byte without moving cursor.
    fn peek_byte(&self) -> Result<u8, CodeEofError>;

    /// Read single bit as a bool value.
    fn read_bool(&mut self) -> Result<bool, CodeEofError> { Ok(self.read_1bit()? == u1::ONE) }
    /// Read single bit.
    fn read_1bit(&mut self) -> Result<u1, CodeEofError>;
    /// Read two bits.
    fn read_2bits(&mut self) -> Result<u2, CodeEofError>;
    /// Read three bits.
    fn read_3bits(&mut self) -> Result<u3, CodeEofError>;
    /// Read four bits.
    fn read_4bits(&mut self) -> Result<u4, CodeEofError>;
    /// Read five bits.
    fn read_5bits(&mut self) -> Result<u5, CodeEofError>;
    /// Read six bits.
    fn read_6bits(&mut self) -> Result<u6, CodeEofError>;
    /// Read seven bits.
    fn read_7bits(&mut self) -> Result<u7, CodeEofError>;

    /// Read byte.
    fn read_byte(&mut self) -> Result<u8, CodeEofError>;
    /// Read word.
    fn read_word(&mut self) -> Result<u16, CodeEofError>;

    /// Read fixed number of bytes and convert it into a result type.
    ///
    /// # Returns
    ///
    /// Resulting data type and a flag for `CK` registry indicating whether it was possible to read
    /// all the data.
    fn read_fixed<N, const LEN: usize>(&mut self, f: impl FnOnce([u8; LEN]) -> N) -> Result<N, CodeEofError>;

    /// Read variable-length byte string.
    ///
    /// # Returns
    ///
    /// Resulting data type and a flag for `CK` registry indicating whether it was possible to read
    /// all the data.
    fn read_bytes(&mut self) -> Result<(SmallBlob, bool), CodeEofError>;

    /// Read external reference id.
    fn read_ref(&mut self) -> Result<Id, CodeEofError>
    where Id: Sized;

    /// Check if the current cursor position is aligned to the next byte.
    ///
    /// # Panics
    ///
    /// If the position is not aligned, panics.
    fn check_aligned(&self);
}

/// Writer converting instructions into a bytecode.
pub trait BytecodeWrite<Id: SiteId> {
    type Error: Error;

    /// Write a single bit from a bool value.
    fn write_bool(&mut self, data: bool) -> Result<(), Self::Error> {
        self.write_1bit(if data { u1::ONE } else { u1::ZERO })
    }

    /// Write a single bit.
    fn write_1bit(&mut self, data: impl Into<u1>) -> Result<(), Self::Error>;
    /// Write two bits.
    fn write_2bits(&mut self, data: impl Into<u2>) -> Result<(), Self::Error>;
    /// Write three bits.
    fn write_3bits(&mut self, data: impl Into<u3>) -> Result<(), Self::Error>;
    /// Write four bits.
    fn write_4bits(&mut self, data: impl Into<u4>) -> Result<(), Self::Error>;
    /// Write five bits.
    fn write_5bits(&mut self, data: impl Into<u5>) -> Result<(), Self::Error>;
    /// Write six bits.
    fn write_6bits(&mut self, data: impl Into<u6>) -> Result<(), Self::Error>;
    /// Write seven bits.
    fn write_7bits(&mut self, data: impl Into<u7>) -> Result<(), Self::Error>;

    /// Write byte.
    fn write_byte(&mut self, data: u8) -> Result<(), Self::Error>;
    /// Write word.
    fn write_word(&mut self, data: u16) -> Result<(), Self::Error>;

    /// Write data representable as a fixed-length byte array.
    fn write_fixed<const LEN: usize>(&mut self, data: [u8; LEN]) -> Result<(), Self::Error>;

    /// Write variable-length byte string.
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Write external reference id.
    fn write_ref(&mut self, id: Id) -> Result<(), Self::Error>;

    /// Check if the current cursor position is aligned to the next byte.
    ///
    /// # Panics
    ///
    /// If the position is not aligned, panics.
    fn check_aligned(&self);
}
