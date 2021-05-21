// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::HashMap;

use crate::registers::Registers;
use crate::{Lib, LibHash, LibSite};

/// AluVM runtime execution environment
#[derive(Getters, Debug)]
pub struct Runtime {
    /// Libraries known to the runtime, identified by their hashes
    libs: HashMap<LibHash, Lib>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    /// A set of registers
    registers: Registers,
}

impl Runtime {
    /// Adds Alu bytecode library to the runtime environment. Returns if the
    /// library was already known.
    pub fn add_lib(&mut self, lib: Lib) -> bool {
        self.libs.insert(lib.lib_hash(), lib).is_none()
    }

    pub fn set_entrypoint(&mut self, entrypoint: LibSite) -> Result<bool, ()> {
        todo!()
    }

    pub fn main(&mut self) -> bool {
        self.call(self.entrypoint)
    }

    pub fn call(&mut self, method: LibSite) -> bool {
        todo!()
    }
}
