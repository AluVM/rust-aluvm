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

//! Alu virtual machine

use alloc::collections::BTreeMap;

use crate::instr::NOp;
use crate::{InstructionSet, Lib, LibId, LibSite, Registers};

/// Error returned by [`Vm::call`] method when the code calls to a library
/// not known to the runtime
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display("call to unknown library {0:#}")]
#[cfg_attr(feature = "std", derive(Error))]
pub struct NoLibraryError(LibId);

/// Alu virtual machine providing single-core execution environment
#[derive(Getters, Debug, Default)]
pub struct Vm<E = NOp>
where
    E: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes
    libs: BTreeMap<LibId, Lib<E>>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    /// A set of registers
    registers: Registers,
}

impl<E> Vm<E>
where
    E: InstructionSet,
{
    /// Constructs new virtual machine with no code in it.
    ///
    /// Calling [`Vm::main`] on it will result in machine termination with `st0` set to `false`.
    pub fn new() -> Vm<E> {
        Vm {
            libs: Default::default(),
            entrypoint: Default::default(),
            registers: Default::default(),
        }
    }

    /// Constructs new virtual machine using the provided library.
    pub fn with(lib: Lib<E>) -> Vm<E> {
        let mut runtime = Vm::new();
        runtime.entrypoint = LibSite::with(0, lib.id());
        runtime.add_lib(lib);
        runtime
    }

    /// Adds Alu bytecode library to the virtual machine.
    ///
    /// # Returns
    ///
    /// `true` if the library was already known and `false` otherwise.
    #[inline]
    pub fn add_lib(&mut self, lib: Lib<E>) -> bool { self.libs.insert(lib.id(), lib).is_none() }

    /// Sets new entry point value (used when calling [`Vm::main`])
    pub fn set_entrypoint(&mut self, entrypoint: LibSite) { self.entrypoint = entrypoint; }

    /// Executes the program starting from the provided entry point (set with [`Vm::set_entrypoint`]
    /// or initialized to 0 offset of the first used library if [`Vm::new`], [`Vm::default`] or
    /// [`Vm::with`] were used).
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    ///
    /// If the code does not has a library matching entry point value does not executes any code and
    /// instantly returns [`NoLibraryError`]. The state of the registers remains unmodified in this
    /// case.
    pub fn main(&mut self) -> Result<bool, NoLibraryError> { self.call(self.entrypoint) }

    /// Executes the program starting from the provided entry point.
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    ///
    /// If the code does not has a library matching entry point value does not executes any code and
    /// instantly returns [`NoLibraryError`]. The state of the registers remains unmodified in this
    /// case.
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
