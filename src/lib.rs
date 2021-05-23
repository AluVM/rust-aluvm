// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg_attr(feature = "std", macro_use)]
extern crate amplify;
#[macro_use]
extern crate bitcoin_hashes;

mod encoding;
pub mod instr;
mod reg;
mod runtime;

pub(crate) use encoding::Cursor;
pub use encoding::CursorError;
pub use instr::{Instr, InstructionSet};
pub use reg::{Reg, Reg16, Reg32, Reg8, RegA, RegBlock, RegR, RegVal, Registers, Value};
pub use runtime::{Blob, Lib, LibHash, LibSite, Runtime};
