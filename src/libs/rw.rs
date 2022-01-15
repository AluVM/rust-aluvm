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

use amplify::num::{u1, u2, u24, u3, u4, u5, u6, u7};

use super::LibId;
use crate::data::Number;
use crate::isa::{Instr, InstructionSet};
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

    /// attempt to write library reference for the lib id {0} which is not a part of libs segment
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
    /// In-place instruction editing
    fn edit<F, E, S>(&mut self, pos: u16, editor: F) -> Result<(), E>
    where
        F: FnOnce(&mut Instr<S>) -> Result<(), E>,
        E: From<CodeEofError>,
        S: InstructionSet;
}
