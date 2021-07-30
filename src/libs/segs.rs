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

//! Data structures representing static library segments

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use core::fmt::{self, Display, Formatter};

use crate::libs::constants::LIBS_SEGMENT_MAX_COUNT;
use crate::libs::{LibId, LibSite};

/// unable to add a library to the library segment: maximum number of libraries (2^16) exceeded
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub struct LibSegOverflow;

mod private {
    pub trait Sealed {}
    impl Sealed for super::LibSeg {}
    impl Sealed for &super::LibSeg {}
}

/// Library segment data keeping collection of libraries which MAY be used in some program.
/// Libraries are referenced in the bytecode using 16-bit position number in this index.
///
/// Library segment keeps ordered collection of [`LibId`] such that the code calling library methods
/// does not need to reference the whole 32-byte id each time and can just provide the library index
/// in the libs segment (2 bytes). Thus, the total number of libraries which can be used by a
/// program is limited to 2^16, and the maximum size of libs segment to 32*2^16 (2 MB).
///
/// NB: The program can reference position outside the scope of the library segment size; in this
///     case VM performs no-operation and sets `st0` to false.
///
/// Libraries MUST be referenced in the libs segment in lexicographic order.
///
/// The implementation MUST ensure that the size of the index never exceeds `u16::MAX`.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct LibSeg {
    /// Set maintains unique library ids which may be iterated in lexicographic ordering
    set: BTreeSet<LibId>,

    /// Table matches lexicographic-based library index to the library id (i.e. this is reverse
    /// index).
    table: BTreeMap<u8, LibId>,
}

impl LibSeg {
    /// Returns iterator over unique libraries iterated in the deterministic (lexicographic) order
    #[inline]
    pub fn iter<'a>(&'a self) -> ::alloc::collections::btree_set::Iter<'a, LibId> {
        (&self).into_iter()
    }
}

impl<'a> IntoIterator for &'a LibSeg {
    type Item = &'a LibId;
    type IntoIter = ::alloc::collections::btree_set::Iter<'a, LibId>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.set.iter() }
}

impl LibSeg {
    /// Constructs libs segment from an iterator over call locations.
    ///
    /// Lib segment deterministically orders library ids according to their [`LibId`] `Ord`
    /// implementation. This is not a requirement, but just a good practice for producing the same
    /// code on different platforms.
    ///
    /// # Error
    ///
    /// Errors with [`LibSegOverflow`] if the number of unique library ids exceeds
    /// [`LIBS_SEGMENT_MAX_COUNT`].
    pub fn from(source: impl IntoIterator<Item = LibSite>) -> Result<Self, LibSegOverflow> {
        LibSeg::with(source.into_iter().map(|site| site.lib))
    }

    /// Constructs libs segment from an iterator over lib ids.
    ///
    /// Lib segment deterministically orders library ids according to their [`LibId`] `Ord`
    /// implementation. This is not a requirement, but just a good practice for producing the same
    /// code on different platforms.
    ///
    /// # Error
    ///
    /// Errors with [`LibSegOverflow`] if the number of unique library ids exceeds
    /// [`LIBS_SEGMENT_MAX_COUNT`].
    pub fn with(source: impl IntoIterator<Item = LibId>) -> Result<Self, LibSegOverflow> {
        let set = source.into_iter().collect::<BTreeSet<LibId>>();
        if set.len() > LIBS_SEGMENT_MAX_COUNT {
            return Err(LibSegOverflow);
        }
        let table = set.iter().enumerate().map(|(index, id)| (index as u8, *id)).collect();
        Ok(LibSeg { set, table })
    }

    /// Returns number of libraries in the lib segment
    #[inline]
    pub fn count(&self) -> u8 { self.set.len() as u8 }

    /// Returns library id with a given index
    #[inline]
    pub fn at(&self, index: u8) -> Option<LibId> { self.table.get(&index).copied() }

    /// Returns index of a library.
    ///
    /// The program can reference position outside the scope of the library segment size; in this
    /// case VM performs no-operation and sets `st0` to false.
    ///
    /// # Returns
    ///
    /// If the library is not present in libs segment, returns `None`.
    #[inline]
    pub fn index(&self, lib: LibId) -> Option<u8> {
        self.set.iter().position(|l| *l == lib).map(|i| i as u8)
    }

    /// Adds library id to the library segment.
    ///
    /// # Errors
    ///
    /// Checks requirement that the total number of libraries must not exceed [`LIBS_MAX_TOTAL`] and
    /// returns [`LibSegOverflow`] otherwise
    ///
    /// # Returns
    ///
    /// `true` if the library was already known and `false` otherwise.
    #[inline]
    pub fn add_lib(&mut self, id: LibId) -> Result<bool, LibSegOverflow> {
        if self.set.len() >= LIBS_SEGMENT_MAX_COUNT {
            Err(LibSegOverflow)
        } else if self.index(id).is_some() {
            Ok(true)
        } else {
            self.set.insert(id);
            let pos = self.index(id).expect("library inserted into a set is absent in the set");
            self.table.insert(pos, id);
            Ok(false)
        }
    }
}

impl Display for LibSeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.set.iter().enumerate().try_for_each(|(line, lib)| {
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
