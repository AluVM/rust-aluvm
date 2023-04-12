// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2023 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023 UBIDECO Institute. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::marker::PhantomData;
use std::collections::btree_map;

use crate::isa::InstructionSet;
use crate::library::constants::LIBS_MAX_TOTAL;
use crate::library::{Lib, LibId, LibSite};

/// Trait for a concrete program implementation provided by a runtime environment.
pub trait Program {
    /// Instruction set architecture used by the program.
    type Isa: InstructionSet;

    /// Iterator type over libraries
    type Iter<'a>: Iterator<Item = &'a Lib>
    where
        Self: 'a;

    /// Returns number of libraries used by the program.
    fn lib_count(&self) -> u16;

    /// Returns an iterator over libraries used by the program.
    fn libs(&self) -> Self::Iter<'_>;

    /// Returns library corresponding to the provided [`LibId`], if the library is known to the
    /// program.
    fn lib(&self, id: LibId) -> Option<&Lib>;

    /// Main entry point into the program.
    fn entrypoint(&self) -> LibSite;
}

/// Errors returned by [`Prog::add_lib`] method
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum ProgError {
    /// ISA id {0} is not supported by the selected instruction set
    IsaNotSupported(String),

    /// Attempt to add library when maximum possible number of libraries is already present in
    /// the VM
    TooManyLibs,
}

/// The most trivial form of a program which is just a collection of libraries with some entry
/// point.
///
/// # Generics
///
/// `RUNTIME_MAX_TOTAL_LIBS`: Maximum total number of libraries supported by a runtime, if it is
/// less than [`LIBS_MAX_TOTAL`]. If the value set is greater than [`LIBS_MAX_TOTAL`] the
/// value is ignored and [`LIBS_MAX_TOTAL`] constant is used instead.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
// #[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
// We need to hardcode generic as a literal, otherwise serde > 1.0.152 fails compilation
pub struct Prog<Isa, const RUNTIME_MAX_TOTAL_LIBS: u16 = 1024>
where
    Isa: InstructionSet,
{
    /// Libraries known to the runtime, identified by their hashes.
    libs: BTreeMap<LibId, Lib>,

    /// Entrypoint for the main function.
    entrypoint: LibSite,

    // #[cfg_attr(feature = "strict_encoding", strict_encoding(skip))]
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom: PhantomData<Isa>,
}

impl<Isa, const RUNTIME_MAX_TOTAL_LIBS: u16> Prog<Isa, RUNTIME_MAX_TOTAL_LIBS>
where
    Isa: InstructionSet,
{
    const RUNTIME_MAX_TOTAL_LIBS: u16 = RUNTIME_MAX_TOTAL_LIBS;

    fn empty_unchecked() -> Self {
        Prog { libs: BTreeMap::new(), entrypoint: LibSite::with(0, zero!()), phantom: default!() }
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
    ) -> Result<Self, ProgError> {
        let mut runtime = Self::empty_unchecked();
        for lib in libs {
            runtime.add_lib(lib)?;
        }
        runtime.set_entrypoint(entrypoint);
        Ok(runtime)
    }

    /// Adds Alu bytecode library to the virtual machine runtime.
    ///
    /// # Errors
    ///
    /// Checks requirement that the total number of libraries must not exceed [`LIBS_MAX_TOTAL`]
    /// and `RUNTIME_MAX_TOTAL_LIBS` - or returns [`ProgError::TooManyLibs`] otherwise.
    ///
    /// Checks that the ISA used by the VM supports ISA extensions specified by the library and
    /// returns [`ProgError::IsaNotSupported`] otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the library was already known and `false` otherwise.
    #[inline]
    pub fn add_lib(&mut self, lib: Lib) -> Result<bool, ProgError> {
        if self.lib_count() >= LIBS_MAX_TOTAL.min(Self::RUNTIME_MAX_TOTAL_LIBS) {
            return Err(ProgError::TooManyLibs);
        }
        for isa in &lib.isae {
            if !Isa::is_supported(isa) {
                return Err(ProgError::IsaNotSupported(isa.to_owned()));
            }
        }
        Ok(self.libs.insert(lib.id(), lib).is_none())
    }

    // TODO: Return error if the library is not known
    /// Sets new entry point value (used when calling [`crate::Vm::run`])
    pub fn set_entrypoint(&mut self, entrypoint: LibSite) { self.entrypoint = entrypoint; }
}

impl<Isa, const RUNTIME_MAX_TOTAL_LIBS: u16> Program for Prog<Isa, RUNTIME_MAX_TOTAL_LIBS>
where
    Isa: InstructionSet,
{
    type Isa = Isa;
    type Iter<'a> = btree_map::Values<'a, LibId, Lib> where Self: 'a;

    fn lib_count(&self) -> u16 { self.libs.len() as u16 }

    fn libs(&self) -> Self::Iter<'_> { self.libs.values() }

    fn lib(&self, id: LibId) -> Option<&Lib> { self.libs.get(&id) }

    fn entrypoint(&self) -> LibSite { self.entrypoint }
}
