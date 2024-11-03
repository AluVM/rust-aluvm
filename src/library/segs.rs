// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Labs,
//     Institute for Distributed and Cognitive Computing, Switzerland.
//     All rights reserved.
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

//! Data structures representing static library segments

#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
use alloc::collections::{btree_set, BTreeSet};
#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt::{self, Debug, Display, Formatter};
use core::str::FromStr;

use amplify::confinement;
use amplify::confinement::Confined;
use strict_encoding::stl::{AlphaCaps, AlphaCapsNum};
use strict_encoding::{InvalidRString, RString};

use crate::library::constants::{
    ISAE_SEGMENT_MAX_COUNT, ISA_ID_MAX_LEN, ISA_ID_MIN_LEN, LIBS_SEGMENT_MAX_COUNT,
};
use crate::library::LibId;
use crate::LIB_NAME_ALUVM;

/// Errors while processing binary-encoded segment data
#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum SegmentError {
    /// the size of the CODE segment is {0}, which exceeds `CODE_SEGMENT_MAX_LEN`
    CodeSegmentTooLarge(usize),

    /// the size of the DATA segment is {0}, which exceeds `DATA_SEGMENT_MAX_LEN`
    DataSegmentTooLarge(usize),

    /// ISA segment error
    #[display(inner)]
    #[from]
    IsaeSegment(IsaSegError),
}

/// Errors while processing ISA extensions segment
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum IsaSegError {
    /// ISA segment is invalid, specifically {0}
    #[from]
    Number(confinement::Error),

    /// ISA id {0} has a wrong name, specifically {1}
    Name(String, InvalidRString),
}

/// ISA extension name.
#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, From)]
#[wrapper(Deref, Display, FromStr)]
#[derive(StrictDumb, StrictType, StrictDecode)]
#[cfg_attr(feature = "std", derive(StrictEncode))]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct IsaName(RString<AlphaCaps, AlphaCapsNum, ISA_ID_MIN_LEN, ISA_ID_MAX_LEN>);

impl_ident_type!(IsaName);
impl_ident_subtype!(IsaName);

/// ISA extensions segment.
#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[derive(StrictType, StrictDecode)]
#[cfg_attr(feature = "std", derive(StrictEncode))]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct IsaSeg(Confined<BTreeSet<IsaName>, 0, ISAE_SEGMENT_MAX_COUNT>);

impl IsaSeg {
    /// Constructs ISAE segment from a string.
    ///
    /// ISAE segment deterministically orders ISAE ids lexicographically. This is not a requirement,
    /// but just a good practice for producing the same code on different platforms.
    ///
    /// # Error
    ///
    /// Errors with [`IsaSegError`] if the segment can't be correctly constructed from the probided
    /// data.
    #[inline]
    pub fn with(s: &'static str) -> Self {
        Self::from_str(s).expect("invalid hardcoded ISA extension name")
    }

    /// Returns number of ISA extensions in the ISAE segment
    #[inline]
    pub fn count(&self) -> u8 { self.0.len() as u8 }

    /// Returns specific ISA id with a given index in the segment.
    #[inline]
    pub fn at(&self, index: u8) -> Option<IsaName> {
        self.0.iter().enumerate().nth(index as usize).map(|(_, isa)| isa).cloned()
    }

    /// Constructs ISA segment from an iterator over ISA extension names.
    #[inline]
    pub fn try_from_iter(
        iter: impl IntoIterator<Item = IsaName>,
    ) -> Result<Self, confinement::Error> {
        Confined::try_from_iter(iter).map(Self)
    }
}

impl IntoIterator for IsaSeg {
    type Item = IsaName;
    type IntoIter = btree_set::IntoIter<IsaName>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl Display for IsaSeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0.iter().map(IsaName::to_string).collect::<Vec<_>>().join(" "))
    }
}

impl FromStr for IsaSeg {
    type Err = IsaSegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut seg = Confined::<BTreeSet<_>, 0, ISAE_SEGMENT_MAX_COUNT>::new();
        for isa in s.split(' ') {
            let name =
                IsaName::from_str(isa).map_err(|err| IsaSegError::Name(isa.to_owned(), err))?;
            seg.push(name)?;
        }
        Ok(Self(seg))
    }
}

/// Library segment data keeping collection of libraries which MAY be used in some program.
/// Libraries are referenced in the bytecode using 16-bit position number in this index.
///
/// Library segment keeps ordered collection of [`LibId`] such that the code calling library methods
/// does not need to reference the whole 32-byte id each time and can just provide the library index
/// in the program segment (1 byte). Thus, the total number of libraries which can be used by a
/// program is limited to 2^8, and the maximum size of program segment to 32*2^8 (8 kB).
///
/// Runtime implementations MUST ensure that the total number of libraries used by a single program
/// do not exceeds [`LIBS_MAX_TOTAL`], limiting the maximum possible total program size for
/// AluVM to ~65 MB.
///
/// NB: The program can reference position outside the scope of the library segment size; in this
///     case VM performs no-operation and sets `st0` to false.
///
/// Libraries MUST be referenced in the program segment in lexicographic order.
///
/// The implementation MUST ensure that the size of the index never exceeds `u16::MAX`.
///
/// [`LIBS_MAX_TOTAL`]: super::constants::LIBS_MAX_TOTAL
#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[derive(StrictType, StrictDecode)]
#[cfg_attr(feature = "std", derive(StrictEncode))]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct LibSeg(Confined<BTreeSet<LibId>, 0, LIBS_SEGMENT_MAX_COUNT>);

impl LibSeg {
    /// Returns iterator over unique libraries iterated in the deterministic (lexicographic) order
    #[inline]
    pub fn iter(&self) -> ::alloc::collections::btree_set::Iter<LibId> { self.into_iter() }
}

impl<'a> IntoIterator for &'a LibSeg {
    type Item = &'a LibId;
    type IntoIter = ::alloc::collections::btree_set::Iter<'a, LibId>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl LibSeg {
    /// Returns number of libraries in the lib segment
    #[inline]
    pub fn count(&self) -> u8 { self.0.len() as u8 }

    /// Returns library id with a given index
    #[inline]
    pub fn at(&self, index: u8) -> Option<LibId> { self.0.iter().nth(index as usize).copied() }

    /// Returns index of a library.
    ///
    /// The program can reference position outside the scope of the library segment size; in this
    /// case VM performs no-operation and sets `st0` to false.
    ///
    /// # Returns
    ///
    /// If the library is not present in program segment, returns `None`.
    #[inline]
    pub fn index(&self, lib: LibId) -> Option<u8> {
        self.0.iter().position(|l| *l == lib).map(|i| i as u8)
    }

    /// Constructs libraries segment from an iterator over library ids.
    #[inline]
    pub fn try_from_iter(
        iter: impl IntoIterator<Item = LibId>,
    ) -> Result<Self, confinement::Error> {
        Confined::try_from_iter(iter).map(Self)
    }
}

impl Display for LibSeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().enumerate().try_for_each(|(line, lib)| {
            writeln!(
                f,
                "{:>2$}{}",
                "",
                lib,
                if line == 0 { 0 } else { f.width().unwrap_or_default() }
            )
        })
    }
}
