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

pub mod instr;
pub mod registers;
mod types;

pub use instr::{Instr, InstructionSet};
#[cfg(feature = "std")]
pub use types::Lib;
pub use types::{Blob, LibHash, LibSite, LiteralParseError, Value};

use crate::registers::Registers;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

#[cfg(feature = "std")]
pub struct Runtime<Extension>
where
    Extension: InstructionSet,
{
    libs: BTreeMap<LibHash, Lib<Extension>>,
    entrypoint: LibSite,
    registers: Registers,
}

#[cfg(feature = "std")]
impl<Extension> Runtime<Extension>
where
    Extension: InstructionSet,
{
    pub fn add_lib(&mut self, lib: Lib<Extension>) -> bool {
        todo!()
        // self.libs.insert(lib.lib_hash(), lib)
    }

    pub fn main(&mut self) -> bool {
        self.call(self.entrypoint)
    }

    pub fn call(&mut self, method: LibSite) -> bool {
        todo!()
    }
}
