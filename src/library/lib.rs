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

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::hash::{Hash as RustHash, Hasher};
use core::str::FromStr;

use amplify::{Bytes32, RawArray};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};

use super::{Cursor, Read};
use crate::data::ByteStr;
use crate::isa::{BytecodeError, ExecStep, InstructionSet};
use crate::library::segs::IsaSeg;
use crate::library::{CodeEofError, LibSeg, LibSegOverflow, SegmentError};
use crate::reg::CoreRegs;
use crate::LIB_NAME_ALUVM;

pub const LIB_ID_TAG: [u8; 32] = *b"urn:ubideco:aluvm:lib:v01#230304";

/// Unique identifier for a library.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug, Display, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[display(Self::to_baid58)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct LibId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl ToBaid58<32> for LibId {
    const HRI: &'static str = "alu";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_raw_array() }
}
impl FromBaid58<32> for LibId {}

impl FromStr for LibId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_baid58_str(s) }
}

impl LibId {
    /// Computes LibId from the provided data
    pub fn with(
        isae: impl AsRef<str>,
        code: impl AsRef<[u8]>,
        data: impl AsRef<[u8]>,
        libs: &LibSeg,
    ) -> LibId {
        let mut hasher = blake3::Hasher::new_keyed(&LIB_ID_TAG);

        let isae = isae.as_ref();
        let code = code.as_ref();
        let data = data.as_ref();
        hasher.update(&(isae.len() as u8).to_le_bytes());
        hasher.update(isae.as_bytes());
        hasher.update(&code.len().to_le_bytes());
        hasher.update(code.as_ref());
        hasher.update(&data.len().to_le_bytes());
        hasher.update(data.as_ref());
        hasher.update(&[libs.count()]);
        for lib in libs {
            hasher.update(lib.as_slice());
        }

        LibId::from_raw_array(hasher.finalize())
    }
}

/// AluVM executable code library
#[derive(Clone, Debug, Default)]
// #[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Lib {
    /// ISA segment
    pub isae: IsaSeg,
    /// Code segment
    pub code: ByteStr,
    /// Data segment
    pub data: ByteStr,
    /// Libs segment
    pub libs: LibSeg,
}

impl Display for Lib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ISAE:   {}", &self.isae)?;
        write!(f, "CODE:\n{:#10}", self.code)?;
        write!(f, "DATA:\n{:#10}", self.data)?;
        write!(f, "LIBS:   {:8}", self.libs)
    }
}

impl PartialEq for Lib {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.id().eq(&other.id()) }
}

impl Eq for Lib {}

impl PartialOrd for Lib {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Lib {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering { self.id().cmp(&other.id()) }
}

impl RustHash for Lib {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) { state.write(&self.id()[..]) }
}

/// Errors while assembling library from the instruction set
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(inner)]
pub enum AssemblerError {
    /// Error assembling code and data segments
    #[from]
    Bytecode(BytecodeError),

    /// Error assembling library segment
    #[from]
    LibSegOverflow(LibSegOverflow),
}

#[cfg(feature = "std")]
impl ::std::error::Error for AssemblerError {
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            AssemblerError::Bytecode(err) => Some(err),
            AssemblerError::LibSegOverflow(err) => Some(err),
        }
    }
}

impl Lib {
    /// Constructs library from raw data split into segments
    pub fn with(
        isa: &str,
        bytecode: Vec<u8>,
        data: Vec<u8>,
        libs: LibSeg,
    ) -> Result<Lib, SegmentError> {
        let isae = IsaSeg::from_iter(isa.split(' '))?;
        Ok(Self {
            isae,
            libs,
            code: ByteStr::try_from(bytecode.borrow())
                .map_err(|_| SegmentError::CodeSegmentTooLarge(bytecode.len()))?,
            data: ByteStr::try_from(data.borrow())
                .map_err(|_| SegmentError::DataSegmentTooLarge(bytecode.len()))?,
        })
    }

    /// Assembles library from the provided instructions by encoding them into bytecode
    pub fn assemble<Isa>(code: &[Isa]) -> Result<Lib, AssemblerError>
    where
        Isa: InstructionSet,
    {
        let call_sites = code.iter().filter_map(|instr| instr.call_site());
        let libs_segment = LibSeg::with(call_sites)?;

        let mut code_segment = ByteStr::default();
        let mut writer = Cursor::<_, ByteStr>::new(&mut code_segment.bytes[..], &libs_segment);
        for instr in code.iter() {
            instr.encode(&mut writer)?;
        }
        let pos = writer.pos();
        let data_segment = writer.into_data_segment();
        code_segment.adjust_len(pos);

        Ok(Lib {
            isae: IsaSeg::from_iter(Isa::isa_ids())
                .expect("ISA instruction set contains incorrect ISAE ids"),
            libs: libs_segment,
            code: code_segment,
            data: data_segment,
        })
    }

    /// Disassembles library into a set of instructions
    pub fn disassemble<Isa>(&self) -> Result<Vec<Isa>, CodeEofError>
    where
        Isa: InstructionSet,
    {
        let mut code = Vec::new();
        let mut reader = Cursor::with(&self.code, &self.data, &self.libs);
        while !reader.is_eof() {
            code.push(Isa::decode(&mut reader)?);
        }
        Ok(code)
    }

    /// Returns hash identifier [`LibId`], representing the library in a unique way.
    ///
    /// Lib ID is computed as SHA256 tagged hash of the serialized library segments (ISAE, code,
    /// data).
    #[inline]
    pub fn id(&self) -> LibId {
        LibId::with(self.isae_segment(), &self.code, &self.data, &self.libs)
    }

    /// Returns ISA data
    #[inline]
    pub fn isae_segment(&self) -> String { self.isae.to_string() }

    /// Returns reference to code segment
    #[inline]
    pub fn code_segment(&self) -> &[u8] { self.code.as_ref() }

    /// Returns reference to data segment
    #[inline]
    pub fn data_segment(&self) -> &[u8] { self.data.as_ref() }

    /// Returns reference to libraries segment
    #[inline]
    pub fn libs_segment(&self) -> &LibSeg { &self.libs }

    /// Executes library code starting at entrypoint
    ///
    /// # Returns
    ///
    /// Location for the external code jump, if any
    pub fn exec<Isa>(&self, entrypoint: u16, registers: &mut CoreRegs) -> Option<LibSite>
    where
        Isa: InstructionSet,
    {
        let mut cursor = Cursor::with(&self.code.bytes[..], &self.data, &self.libs);
        let lib_hash = self.id();
        cursor.seek(entrypoint).ok()?;

        while !cursor.is_eof() {
            let pos = cursor.pos();

            let instr = Isa::decode(&mut cursor).ok()?;
            let next = instr.exec(registers, LibSite::with(pos, lib_hash));

            #[cfg(all(debug_assertions, feature = "std"))]
            eprint!("\n@{:06}> {:48}; st0={}", pos, instr, registers.st0);

            if !registers.acc_complexity(instr) {
                #[cfg(all(debug_assertions, feature = "std"))]
                eprintln!();
                return None;
            }
            match next {
                ExecStep::Stop => {
                    #[cfg(all(debug_assertions, feature = "std"))]
                    eprintln!();
                    return None;
                }
                ExecStep::Next => continue,
                ExecStep::Jump(pos) => {
                    #[cfg(all(debug_assertions, feature = "std"))]
                    eprint!(" -> {}", pos);
                    cursor.seek(pos).ok()?;
                }
                ExecStep::Call(site) => {
                    #[cfg(all(debug_assertions, feature = "std"))]
                    eprint!(" -> {}", site);
                    return Some(site);
                }
            }
        }

        None
    }
}

/// Location within a library
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Display)]
// #[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[display("{pos} @ {lib}")]
pub struct LibSite {
    /// Library hash
    pub lib: LibId,

    /// Offset from the beginning of the code, in bytes
    pub pos: u16,
}

impl LibSite {
    /// Constricts library site reference from a given position and library hash
    /// value
    pub fn with(pos: u16, lib: LibId) -> LibSite { LibSite { lib, pos } }
}
