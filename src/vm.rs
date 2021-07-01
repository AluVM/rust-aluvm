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

use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use crate::isa::{InstructionSet, ReservedOp};
use crate::libs::{Lib, LibId, LibSite};
use crate::reg::CoreRegs;

/// Alu virtual machine providing single-core execution environment
#[derive(Getters, Debug, Default)]
pub struct Vm<E = ReservedOp>
where
    E: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes
    libs: BTreeMap<LibId, Lib<E>>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    /// A set of registers
    registers: Box<CoreRegs>,
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
    pub fn main(&mut self) -> bool { self.call(self.entrypoint) }

    /// Executes the program starting from the provided entry point.
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    pub fn call(&mut self, method: LibSite) -> bool {
        let mut call = Some(method);
        while let Some(ref mut site) = call {
            if let Some(lib) = self.libs.get(&site.lib) {
                call = lib.run(site.pos, &mut self.registers);
            } else if let Some(pos) = site.pos.checked_add(1) {
                site.pos = pos;
            } else {
                call = None;
            };
        }
        self.registers.st0
    }
}
