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

use amplify_num::{u1, u2, u24, u3, u4, u5, u6, u7};

use super::LibId;
use crate::data::Number;
use crate::reg::RegisterFamily;

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

pub trait Read: private::Sealed {
    fn is_end(&self) -> bool;
    fn peek_u8(&self) -> Result<u8, CodeEofError>;
    fn read_bool(&mut self) -> Result<bool, CodeEofError>;
    fn read_u1(&mut self) -> Result<u1, CodeEofError>;
    fn read_u2(&mut self) -> Result<u2, CodeEofError>;
    fn read_u3(&mut self) -> Result<u3, CodeEofError>;
    fn read_u4(&mut self) -> Result<u4, CodeEofError>;
    fn read_u5(&mut self) -> Result<u5, CodeEofError>;
    fn read_u6(&mut self) -> Result<u6, CodeEofError>;
    fn read_u7(&mut self) -> Result<u7, CodeEofError>;
    fn read_u8(&mut self) -> Result<u8, CodeEofError>;
    fn read_u16(&mut self) -> Result<u16, CodeEofError>;
    fn read_i16(&mut self) -> Result<i16, CodeEofError>;
    fn read_u24(&mut self) -> Result<u24, CodeEofError>;
    fn read_lib(&mut self) -> Result<LibId, CodeEofError>;
    fn read_data(&mut self) -> Result<(&[u8], bool), CodeEofError>;
    fn read_number(&mut self, reg: impl RegisterFamily) -> Result<Number, CodeEofError>;
}

pub trait Write: private::Sealed {
    fn write_bool(&mut self, data: bool) -> Result<(), WriteError>;
    fn write_u1(&mut self, data: impl Into<u1>) -> Result<(), WriteError>;
    fn write_u2(&mut self, data: impl Into<u2>) -> Result<(), WriteError>;
    fn write_u3(&mut self, data: impl Into<u3>) -> Result<(), WriteError>;
    fn write_u4(&mut self, data: impl Into<u4>) -> Result<(), WriteError>;
    fn write_u5(&mut self, data: impl Into<u5>) -> Result<(), WriteError>;
    fn write_u6(&mut self, data: impl Into<u6>) -> Result<(), WriteError>;
    fn write_u7(&mut self, data: impl Into<u7>) -> Result<(), WriteError>;
    fn write_u8(&mut self, data: impl Into<u8>) -> Result<(), WriteError>;
    fn write_u16(&mut self, data: impl Into<u16>) -> Result<(), WriteError>;
    fn write_i16(&mut self, data: impl Into<i16>) -> Result<(), WriteError>;
    fn write_u24(&mut self, data: impl Into<u24>) -> Result<(), WriteError>;
    fn write_lib(&mut self, data: LibId) -> Result<(), WriteError>;
    fn write_data(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), WriteError>;
    fn write_number(&mut self, reg: impl RegisterFamily, value: Number) -> Result<(), WriteError>;
}
