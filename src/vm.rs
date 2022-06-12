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

//! Alu virtual machine

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::marker::PhantomData;

use crate::isa::{Instr, InstructionSet, ReservedOp};
use crate::libs::constants::LIBS_MAX_TOTAL;
use crate::libs::{Lib, LibId, LibSite};
use crate::reg::CoreRegs;

/// Errors returned by [`Vm::add_lib`] method
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum Error {
    /// ISA id {0} is not supported by the selected instruction set
    IsaNotSupported(String),

    /// Attempt to add library when maximum possible number of libraries is already present in the
    /// VM
    TooManyLibs,
}

/// Alu virtual machine providing single-core execution environment
#[derive(Getters, Debug)]
pub struct Vm<Isa = Instr<ReservedOp>>
where
    Isa: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes
    libs: BTreeMap<LibId, Lib>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    /// A set of registers
    registers: Box<CoreRegs>,

    phantom: PhantomData<Isa>,
}

impl<Isa> Vm<Isa>
where
    Isa: InstructionSet,
{
    /// Constructs new virtual machine using provided single library.
    pub fn new(lib: Lib) -> Vm<Isa> {
        let mut runtime = Vm {
            libs: bmap! {},
            entrypoint: LibSite::with(0, lib.id()),
            registers: default!(),
            phantom: default!(),
        };
        runtime.add_lib(lib).expect("adding single library to lib segment overflows");
        runtime
    }

    /// Constructs new virtual machine from a set of libraries with a given entry point.
    pub fn with(
        libs: impl IntoIterator<Item = Lib>,
        entrypoint: LibSite,
    ) -> Result<Vm<Isa>, Error> {
        let mut runtime =
            Vm { libs: bmap! {}, entrypoint, registers: default!(), phantom: default!() };
        for lib in libs {
            runtime.add_lib(lib)?;
        }
        Ok(runtime)
    }

    /// Adds Alu bytecode library to the virtual machine.
    ///
    /// # Errors
    ///
    /// Checks requirement that the total number of libraries must not exceed [`LIBS_MAX_TOTAL`] and
    /// returns [`Error::TooManyLibs`] otherwise.
    ///
    /// Checks that the ISA used by the VM supports ISA extensions specified by the library and
    /// returns [`Error::IsaNotSupported`] otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the library was already known and `false` otherwise.
    #[inline]
    pub fn add_lib(&mut self, lib: Lib) -> Result<bool, Error> {
        if self.libs.len() >= LIBS_MAX_TOTAL {
            return Err(Error::TooManyLibs);
        }
        for isa in &lib.isae {
            if !Isa::is_supported(isa) {
                return Err(Error::IsaNotSupported(isa.to_owned()));
            }
        }
        Ok(self.libs.insert(lib.id(), lib).is_none())
    }

    /// Sets new entry point value (used when calling [`Vm::main`])
    pub fn set_entrypoint(&mut self, entrypoint: LibSite) { self.entrypoint = entrypoint; }

    /// Executes the program starting from the provided entry point (set with [`Vm::set_entrypoint`]
    /// and [`Vm::with`], or initialized to 0 offset of the first used library if [`Vm::new`] was
    /// used).
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
                call = lib.run::<Isa>(site.pos, &mut self.registers);
            } else if let Some(pos) = site.pos.checked_add(1) {
                site.pos = pos;
            } else {
                call = None;
            };
        }
        self.registers.st0
    }
}
