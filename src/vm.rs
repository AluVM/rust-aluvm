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

use alloc::collections::BTreeMap;

use crate::instr::NOp;
use crate::{InstructionSet, Lib, LibHash, LibSite, Registers};

/// Error returned by [`Vm::call`] method when the code calls to a library
/// not known to the runtime
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display("call to unknown library {0:#}")]
#[cfg_attr(feature = "std", derive(Error))]
pub struct NoLibraryError(LibHash);

/// Alu virtual machine providing single-core execution environment
#[derive(Getters, Debug, Default)]
pub struct Vm<E = NOp>
where
    E: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes
    libs: BTreeMap<LibHash, Lib<E>>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    /// A set of registers
    registers: Registers,
}

impl<E> Vm<E>
where
    E: InstructionSet,
{
    pub fn new() -> Vm<E> {
        Vm {
            libs: Default::default(),
            entrypoint: Default::default(),
            registers: Default::default(),
        }
    }

    pub fn with(lib: Lib<E>) -> Vm<E> {
        let mut runtime = Vm::new();
        runtime.entrypoint = LibSite::with(0, lib.lib_hash());
        runtime.add_lib(lib);
        runtime
    }

    /// Adds Alu bytecode library to the virtual machine.
    ///
    /// # Returns
    ///
    /// `true` if the library was already known and `false` otherwise.
    #[inline]
    pub fn add_lib(&mut self, lib: Lib<E>) -> bool {
        self.libs.insert(lib.lib_hash(), lib).is_none()
    }

    pub fn set_entrypoint(&mut self, entrypoint: LibSite) { self.entrypoint = entrypoint; }

    pub fn main(&mut self) -> Result<bool, NoLibraryError> { self.call(self.entrypoint) }

    pub fn call(&mut self, mut method: LibSite) -> Result<bool, NoLibraryError> {
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
