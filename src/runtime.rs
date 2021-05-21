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

use crate::instr::Nop;
use crate::registers::Registers;
use crate::{InstructionSet, Lib, LibHash, LibSite};

/// Error returned by [`Runtime::call`] method when the code calls to a library
/// not known to the runtime
#[derive(
    Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error,
)]
#[display("call to unknown library {0:#}")]
pub struct NoLibraryError(LibHash);

/// AluVM runtime execution environment
#[derive(Getters, Debug, Default)]
pub struct Runtime<E = Nop>
where
    E: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes
    libs: HashMap<LibHash, Lib<E>>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    /// A set of registers
    registers: Registers,
}

impl<E> Runtime<E>
where
    E: InstructionSet,
{
    pub fn new() -> Runtime<E> {
        Runtime {
            libs: Default::default(),
            entrypoint: Default::default(),
            registers: Default::default(),
        }
    }

    pub fn with(lib: Lib<E>) -> Runtime<E> {
        let mut runtime = Runtime::new();
        runtime.add_lib(lib);
        runtime
    }

    /// Adds Alu bytecode library to the runtime environment. Returns if the
    /// library was already known.
    pub fn add_lib(&mut self, lib: Lib<E>) -> bool {
        self.libs.insert(lib.lib_hash(), lib).is_none()
    }

    pub fn set_entrypoint(&mut self, entrypoint: LibSite) {
        self.entrypoint = entrypoint;
    }

    pub fn main(&mut self) -> Result<bool, NoLibraryError> {
        self.call(self.entrypoint)
    }

    pub fn call(
        &mut self,
        mut method: LibSite,
    ) -> Result<bool, NoLibraryError> {
        while let Some(m) = self
            .libs
            .get(&method.lib)
            .ok_or(NoLibraryError(method.lib))?
            .run(method.pos, &mut self.registers)
        {
            method = m
        }
        Ok(self.registers.st0)
    }
}
