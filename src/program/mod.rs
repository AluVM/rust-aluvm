// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://github.com/internet2-org/aluvm-spec>
//
// Designed & written in 2021-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

//! Business logic and data structures for working with AluVM code libraries

pub mod constants;
mod cursor;
mod lib;
mod program;
mod rw;
mod segs;

pub use cursor::Cursor;
pub use lib::{AssemblerError, Lib, LibId, LibIdError, LibIdTag, LibSite};
pub use program::{LibError, Program};
pub use rw::{CodeEofError, Read, Write, WriteError};
pub use segs::{IsaSeg, IsaSegError, LibSeg, LibSegOverflow, SegmentError};
