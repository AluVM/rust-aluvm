// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 LNP/BP Standards Association, Switzerland.
// Copyright (C) 2024-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2021-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use core::fmt;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

use amplify::confinement::{SmallBlob, TinyOrdSet};
use amplify::Bytes32;
use baid64::{Baid64ParseError, DisplayBaid64, FromBaid64Str};
use commit_verify::{CommitId, CommitmentId, Digest, Sha256};
use strict_encoding::{StrictDeserialize, StrictSerialize};

use crate::core::SiteId;
use crate::{IsaId, Site, LIB_NAME_ALUVM};

pub const LIB_ID_TAG: &str = "urn:ubideco:aluvm:lib:v01#241020";

/// Unique identifier for an AluVM library.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct LibId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl SiteId for LibId {}

impl CommitmentId for LibId {
    const TAG: &'static str = LIB_ID_TAG;
}

impl DisplayBaid64 for LibId {
    const HRI: &'static str = "alu";
    const CHUNKING: bool = true;
    const PREFIX: bool = true;
    const EMBED_CHECKSUM: bool = false;
    const MNEMONIC: bool = true;
    fn to_baid64_payload(&self) -> [u8; 32] { self.to_byte_array() }
}
impl FromBaid64Str for LibId {}
impl FromStr for LibId {
    type Err = Baid64ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_baid64_str(s) }
}
impl Display for LibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { self.fmt_baid64(f) }
}

impl From<Sha256> for LibId {
    fn from(hash: Sha256) -> Self { Self(Bytes32::from_byte_array(hash.finalize())) }
}

/// Location inside the instruction sequence which can be executed by the core.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub struct LibSite {
    pub lib_id: LibId,
    pub offset: u16,
}

impl From<Site<LibId>> for LibSite {
    fn from(site: Site<LibId>) -> Self { Self { lib_id: site.prog_id, offset: site.offset } }
}

impl LibSite {
    #[inline]
    pub fn new(lib_id: LibId, offset: u16) -> Self { LibSite { lib_id, offset } }
}

pub type LibsSeg = TinyOrdSet<LibId>;

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[derive(CommitEncode)]
#[commit_encode(id = LibId, strategy = strict)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Lib {
    pub isae: TinyOrdSet<IsaId>,
    pub code: SmallBlob,
    pub data: SmallBlob,
    pub libs: LibsSeg,
}

impl StrictSerialize for Lib {}
impl StrictDeserialize for Lib {}

impl Lib {
    pub fn lib_id(&self) -> LibId { self.commit_id() }

    pub fn isae_string(&self) -> String {
        self.isae
            .iter()
            .map(IsaId::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Display for Lib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ISAE:  {}", self.isae_string())?;
        writeln!(f, "CODE: {:x}", self.code)?;
        writeln!(f, "DATA: {:x}", self.data)?;
        if self.libs.len() > 0 {
            writeln!(
                f,
                "LIBS: {:8}",
                self.libs
                    .iter()
                    .map(LibId::to_string)
                    .collect::<Vec<_>>()
                    .join("\n        ")
            )
        } else {
            writeln!(f, "LIBS: ~")
        }
    }
}

#[cfg(test)]
mod test {
    use strict_encoding::StrictDumb;

    use super::*;

    #[test]
    fn lib_id_display() {
        let id = Lib::strict_dumb().lib_id();
        assert_eq!(
            format!("{id}"),
            "alu:uZkzX1J9-i5EvGTf-J1TB79p-OBvKq5x-1U2n4qd-8Nso3Ag#reunion-cable-tractor"
        );
        assert_eq!(
            format!("{id:-}"),
            "uZkzX1J9-i5EvGTf-J1TB79p-OBvKq5x-1U2n4qd-8Nso3Ag#reunion-cable-tractor"
        );
        assert_eq!(format!("{id:#}"), "alu:uZkzX1J9-i5EvGTf-J1TB79p-OBvKq5x-1U2n4qd-8Nso3Ag");
        assert_eq!(format!("{id:-#}"), "uZkzX1J9-i5EvGTf-J1TB79p-OBvKq5x-1U2n4qd-8Nso3Ag");
    }

    #[test]
    fn lib_id_from_str() {
        let id = Lib::strict_dumb().lib_id();
        assert_eq!(
            id,
            LibId::from_str(
                "alu:uZkzX1J9-i5EvGTf-J1TB79p-OBvKq5x-1U2n4qd-8Nso3Ag#reunion-cable-tractor"
            )
            .unwrap()
        );
        assert_eq!(id, LibId::from_str("alu:uZkzX1J9i5EvGTfJ1TB79pOBvKq5x1U2n4qd8Nso3Ag").unwrap());
        assert_eq!(
            id,
            LibId::from_str(
                "alu:uZkzX1J9i5EvGTfJ1TB79pOBvKq5x1U2n4qd8Nso3Ag#reunion-cable-tractor"
            )
            .unwrap()
        );

        assert_eq!(id, LibId::from_str("uZkzX1J9i5EvGTfJ1TB79pOBvKq5x1U2n4qd8Nso3Ag").unwrap());
    }
}
