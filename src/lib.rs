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

// TODO: Complete arithmetic integer implementation
// TODO: Complete arithmetic float implementation
// TODO: Complete cycle-shit operations
// TODO: Add float registers
// TODO: Refactor arithmetic operations according to the new spec
// TODO: Refactor bitwise operation arguments
// TODO: Implement string operations
// TODO: Implement cryptorgraphic operations
// TODO: Complete assembly compiler
// TODO: Add additional operations for checking
// TODO: Add overflow register
// TODO: Support assembly compillation in `no_std` environments

extern crate alloc;

#[macro_use]
extern crate amplify_derive;

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
