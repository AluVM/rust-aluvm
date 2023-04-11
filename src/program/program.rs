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

use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::marker::PhantomData;

use super::constants::LIBS_MAX_TOTAL;
use super::*;
use crate::isa::InstructionSet;

/// Errors returned by [`Program::add_lib`] method
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum LibError {
    /// ISA id {0} is not supported by the selected instruction set
    IsaNotSupported(String),

    /// Attempt to add library when maximum possible number of libraries is already present in
    /// the VM
    TooManyLibs,
}

/// An AluVM program executable by a virtual machine.
///
/// # Generics
///
/// `RUNTIME_MAX_TOTAL_LIBS`: Maximum total number of libraries supported by a runtime, if it is
/// less than [`LIBS_MAX_TOTAL`]. If the value set is greater than [`LIBS_MAX_TOTAL`] the
/// value is ignored and [`LIBS_MAX_TOTAL`] constant is used instead.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
// Removed due to a bug in new serde which makes it unable to work with generic defaults
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Program<Isa, const RUNTIME_MAX_TOTAL_LIBS: u16 = LIBS_MAX_TOTAL>
where
    Isa: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes
    libs: BTreeMap<LibId, Lib>,

    /// Entrypoint for the main function
    entrypoint: LibSite,

    #[cfg_attr(feature = "strict_encoding", strict_encoding(skip))]
    // #[cfg_attr(feature = "serde", serde(skip))]
    phantom: PhantomData<Isa>,
}

impl<Isa, const RUNTIME_MAX_TOTAL_LIBS: u16> Program<Isa, RUNTIME_MAX_TOTAL_LIBS>
where
    Isa: InstructionSet,
{
    const RUNTIME_MAX_TOTAL_LIBS: u16 = RUNTIME_MAX_TOTAL_LIBS;

    fn empty_unchecked() -> Self {
        Program {
            libs: BTreeMap::new(),
            entrypoint: LibSite::with(0, zero!()),
            phantom: default!(),
        }
    }

    /// Constructs new virtual machine runtime using provided single library. Entry point is set
    /// to zero offset by default.
    pub fn new(lib: Lib) -> Self {
        let mut runtime = Self::empty_unchecked();
        let id = lib.id();
        runtime.add_lib(lib).expect("adding single library to lib segment overflows");
        runtime.set_entrypoint(LibSite::with(0, id));
        runtime
    }

    /// Constructs new virtual machine runtime from a set of libraries with a given entry point.
    pub fn with(
        libs: impl IntoIterator<Item = Lib>,
        entrypoint: LibSite,
    ) -> Result<Self, LibError> {
        let mut runtime = Self::empty_unchecked();
        for lib in libs {
            runtime.add_lib(lib)?;
        }
        runtime.set_entrypoint(entrypoint);
        Ok(runtime)
    }

    /// Returns reference to a specific library, if it is part of the current program.
    pub fn lib(&self, id: LibId) -> Option<&Lib> { self.libs.get(&id) }

    /// Adds Alu bytecode library to the virtual machine runtime.
    ///
    /// # Errors
    ///
    /// Checks requirement that the total number of libraries must not exceed [`LIBS_MAX_TOTAL`]
    /// and `RUNTIME_MAX_TOTAL_LIBS` - or returns [`LibError::TooManyLibs`] otherwise.
    ///
    /// Checks that the ISA used by the VM supports ISA extensions specified by the library and
    /// returns [`LibError::IsaNotSupported`] otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the library was already known and `false` otherwise.
    #[inline]
    pub fn add_lib(&mut self, lib: Lib) -> Result<bool, LibError> {
        if self.libs_count() >= LIBS_MAX_TOTAL.min(Self::RUNTIME_MAX_TOTAL_LIBS) {
            return Err(LibError::TooManyLibs);
        }
        for isa in &lib.isae {
            if !Isa::is_supported(isa) {
                return Err(LibError::IsaNotSupported(isa.to_owned()));
            }
        }
        Ok(self.libs.insert(lib.id(), lib).is_none())
    }

    /// Returns number of libraries used by the program.
    pub fn libs_count(&self) -> u16 { self.libs.len() as u16 }

    /// Returns program entry point.
    pub fn entrypoint(&self) -> LibSite { self.entrypoint }

    // TODO: Return error if the library is not known
    /// Sets new entry point value (used when calling [`crate::Vm::run`])
    pub fn set_entrypoint(&mut self, entrypoint: LibSite) { self.entrypoint = entrypoint; }
}
