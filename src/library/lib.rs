// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Institute. All rights reserved.
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

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::{String, ToString};
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::{self, Display, Formatter};
use core::hash::{Hash as RustHash, Hasher};
use core::str::FromStr;

use amplify::confinement::SmallBlob;
use amplify::{confinement, ByteArray, Bytes32};
use baid64::{Baid64ParseError, DisplayBaid64, FromBaid64Str};
use sha2::{Digest, Sha256};
use strict_encoding::{StrictDeserialize, StrictSerialize};

#[cfg(feature = "ascii-armor")]
pub use self::_armor::LibArmorError;
use super::{Cursor, Read, WriteError};
use crate::data::ByteStr;
use crate::isa::{Bytecode, BytecodeError, ExecStep, Instr, InstructionSet};
use crate::library::segs::IsaSeg;
use crate::library::{CodeEofError, LibSeg, SegmentError};
use crate::reg::CoreRegs;
use crate::LIB_NAME_ALUVM;

pub const LIB_ID_TAG: [u8; 32] = *b"urn:ubideco:aluvm:lib:v01#230304";

/// Unique identifier for a library.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictDecode)]
#[cfg_attr(feature = "std", derive(StrictEncode))]
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

impl LibId {
    /// Computes LibId from the provided data
    pub fn with(
        isae: impl AsRef<str>,
        code: impl AsRef<[u8]>,
        data: impl AsRef<[u8]>,
        libs: &LibSeg,
    ) -> LibId {
        let mut tagger = Sha256::default();
        tagger.update(LIB_ID_TAG);
        let tag = tagger.finalize();

        let mut hasher = Sha256::default();
        hasher.update(tag);
        hasher.update(tag);

        let isae = isae.as_ref();
        let code = code.as_ref();
        let data = data.as_ref();
        hasher.update((isae.len() as u8).to_le_bytes());
        hasher.update(isae.as_bytes());
        hasher.update((code.len() as u16).to_le_bytes());
        hasher.update(code);
        hasher.update((data.len() as u16).to_le_bytes());
        hasher.update(data);
        hasher.update([libs.count()]);
        for lib in libs {
            hasher.update(lib.as_slice());
        }

        LibId::from_byte_array(hasher.finalize())
    }
}

/// AluVM executable code library
#[derive(Clone, Debug, Default)]
#[derive(StrictType, StrictDecode)]
#[cfg_attr(feature = "std", derive(StrictEncode))]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Lib {
    /// ISA segment
    pub isae: IsaSeg,
    /// Code segment
    pub code: SmallBlob,
    /// Data segment
    pub data: SmallBlob,
    /// Libs segment
    pub libs: LibSeg,
}

impl StrictSerialize for Lib {}
impl StrictDeserialize for Lib {}

impl Display for Lib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ISAE:   {}", &self.isae)?;
        write!(f, "CODE:\n{:#10}", ByteStr::with(self.code.as_ref()))?;
        write!(f, "DATA:\n{:#10}", ByteStr::with(self.data.as_ref()))?;
        if self.libs.count() > 0 {
            write!(f, "LIBS:   {:8}", self.libs)
        } else {
            write!(f, "LIBS:   none")
        }
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

#[cfg(feature = "ascii-armor")]
mod _armor {
    use amplify::confinement::{self, Confined, U24 as U24MAX};
    use armor::{ArmorHeader, ArmorParseError, AsciiArmor, ASCII_ARMOR_ID};
    use strict_encoding::DeserializeError;

    use super::*;

    const ASCII_ARMOR_ISAE: &str = "ISA-Extensions";
    const ASCII_ARMOR_DEPENDENCY: &str = "Dependency";

    /// Errors while deserializing library from an ASCII Armor.
    #[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
    #[display(inner)]
    pub enum LibArmorError {
        /// Armor parse error.
        #[from]
        Armor(ArmorParseError),

        /// The provided data exceed maximum possible library size.
        #[from(confinement::Error)]
        TooLarge,

        /// Library data deserialization error.
        #[from]
        Decode(DeserializeError),
    }

    impl AsciiArmor for Lib {
        type Err = LibArmorError;
        const PLATE_TITLE: &'static str = "ALUVM LIB";

        fn ascii_armored_headers(&self) -> Vec<ArmorHeader> {
            let mut headers = vec![
                ArmorHeader::new(ASCII_ARMOR_ID, self.id().to_string()),
                ArmorHeader::new(ASCII_ARMOR_ISAE, self.isae.to_string()),
            ];
            for dep in &self.libs {
                headers.push(ArmorHeader::new(ASCII_ARMOR_DEPENDENCY, dep.to_string()));
            }
            headers
        }

        fn to_ascii_armored_data(&self) -> Vec<u8> {
            self.to_strict_serialized::<U24MAX>().expect("type guarantees").to_vec()
        }

        fn with_headers_data(_headers: Vec<ArmorHeader>, data: Vec<u8>) -> Result<Self, Self::Err> {
            // TODO: check id, dependencies and ISAE
            let data = Confined::try_from(data)?;
            let me = Self::from_strict_serialized::<U24MAX>(data)?;
            Ok(me)
        }
    }
}

/// Errors while assembling library from the instruction set
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(inner)]
pub enum AssemblerError {
    /// Error assembling code and data segments
    #[from]
    #[from(WriteError)]
    Bytecode(BytecodeError),

    /// Error assembling library segment
    #[from]
    LibSegOverflow(confinement::Error),
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
        let isae = IsaSeg::from_str(isa)?;
        let len = bytecode.len();
        Ok(Self {
            isae,
            libs,
            code: SmallBlob::try_from(bytecode)
                .map_err(|_| SegmentError::CodeSegmentTooLarge(len))?,
            data: SmallBlob::try_from(data).map_err(|_| SegmentError::DataSegmentTooLarge(len))?,
        })
    }

    /// Assembles library from the provided instructions by encoding them into bytecode
    pub fn assemble<Instr>(code: &[Instr], isae: IsaSeg) -> Result<Lib, AssemblerError>
    where
        Instr: Bytecode,
    {
        let call_sites = code.iter().filter_map(|instr| instr.call_site()).map(|site| site.lib);
        let libs_segment = LibSeg::try_from_iter(call_sites)?;

        let mut code_segment = ByteStr::default();
        let mut writer = Cursor::<_, ByteStr>::new(&mut code_segment.bytes[..], &libs_segment);
        for instr in code.iter() {
            instr.encode(&mut writer)?;
        }
        let pos = writer.pos();
        let data_segment = SmallBlob::from_collection_unsafe(writer.into_data_segment().to_vec());
        code_segment.adjust_len(pos);
        let code_segment = SmallBlob::from_collection_unsafe(code_segment.to_vec());

        Ok(Lib { isae, libs: libs_segment, code: code_segment, data: data_segment })
    }

    /// Disassembles library into a set of instructions
    pub fn disassemble<Isa>(&self) -> Result<Vec<Isa>, CodeEofError>
    where
        Isa: Bytecode,
    {
        let mut code = Vec::new();
        let mut reader = Cursor::with(&self.code, &self.data, &self.libs);
        while !reader.is_eof() {
            code.push(Isa::decode(&mut reader)?);
        }
        Ok(code)
    }

    /// Disassembles library into a set of instructions and offsets and prints it to the writer.
    #[cfg(feature = "std")]
    pub fn print_disassemble<Isa>(
        &self,
        mut writer: impl std::io::Write,
    ) -> Result<(), std::io::Error>
    where
        Isa: InstructionSet,
    {
        let mut reader = Cursor::with(&self.code, &self.data, &self.libs);
        while !reader.is_eof() {
            let pos = reader.offset().0 as usize;
            write!(writer, "offset_0x{pos:04X}: ")?;
            match Instr::<Isa>::decode(&mut reader) {
                Ok(instr) => writeln!(writer, "{instr}")?,
                Err(_) => writeln!(writer, "\n{}", ByteStr::with(&self.code.as_ref()[pos..]))?,
            }
        }
        Ok(())
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
    pub fn exec<Isa>(
        &self,
        entrypoint: u16,
        registers: &mut CoreRegs,
        context: &Isa::Context<'_>,
    ) -> Option<LibSite>
    where
        Isa: InstructionSet,
    {
        #[cfg(feature = "log")]
        let (m, w, d, g, r, y, z) = (
            "\x1B[0;35m",
            "\x1B[1;1m",
            "\x1B[0;37;2m",
            "\x1B[0;32m",
            "\x1B[0;31m",
            "\x1B[0;33m",
            "\x1B[0m",
        );

        let mut cursor = Cursor::with(&self.code, &self.data, &self.libs);
        let lib_hash = self.id();
        cursor.seek(entrypoint).ok()?;

        #[cfg(feature = "log")]
        let mut st0 = registers.st0;

        while !cursor.is_eof() {
            let pos = cursor.pos();

            let instr = Isa::decode(&mut cursor).ok()?;

            #[cfg(feature = "log")]
            {
                eprint!("{m}@{pos:06}:{z} {: <32}; ", instr.to_string());
                for reg in instr.src_regs() {
                    let val = registers.get(reg);
                    eprint!("{d}{reg}={z}{w}{val}{z} ");
                }
            }

            let next = instr.exec(registers, LibSite::with(pos, lib_hash), context);

            #[cfg(feature = "log")]
            {
                eprint!("-> ");
                for reg in instr.dst_regs() {
                    let val = registers.get(reg);
                    eprint!("{g}{reg}={y}{val}{z} ");
                }
                if st0 != registers.st0 {
                    let c = if registers.st0 { g } else { r };
                    eprint!(" {d}st0={z}{c}{}{z} ", registers.st0);
                }

                st0 = registers.st0;
            }

            if !registers.acc_complexity(instr) {
                #[cfg(feature = "log")]
                eprintln!("complexity overflow");
                return None;
            }
            match next {
                ExecStep::Stop => {
                    #[cfg(feature = "log")]
                    {
                        let c = if registers.st0 { g } else { r };
                        eprintln!("execution stopped; {d}st0={z}{c}{}{z}", registers.st0);
                    }
                    return None;
                }
                ExecStep::Next => {
                    #[cfg(feature = "log")]
                    eprintln!();
                    continue;
                }
                ExecStep::Jump(pos) => {
                    #[cfg(feature = "log")]
                    eprintln!("{}", pos);
                    cursor.seek(pos).ok()?;
                }
                ExecStep::Call(site) => {
                    #[cfg(feature = "log")]
                    eprintln!("{}", site);
                    return Some(site);
                }
            }
        }

        None
    }
}

/// Location within a library
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Display)]
#[derive(StrictType, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM)]
#[cfg_attr(feature = "std", derive(StrictEncode))]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lib_id_display() {
        let id = LibId::with("FLOAT", b"", b"", &none!());
        assert_eq!(
            format!("{id}"),
            "alu:650XHPmh-WpXWR5R-Uz4B5jX-jeDqcyr-HXpdZxY-aX9gfO4#plasma-tunnel-mama"
        );
        assert_eq!(
            format!("{id:-}"),
            "650XHPmh-WpXWR5R-Uz4B5jX-jeDqcyr-HXpdZxY-aX9gfO4#plasma-tunnel-mama"
        );
        assert_eq!(format!("{id:#}"), "alu:650XHPmh-WpXWR5R-Uz4B5jX-jeDqcyr-HXpdZxY-aX9gfO4");
        assert_eq!(format!("{id:-#}"), "650XHPmh-WpXWR5R-Uz4B5jX-jeDqcyr-HXpdZxY-aX9gfO4");
    }

    #[test]
    fn lib_id_from_str() {
        let id = LibId::with("FLOAT", b"", b"", &none!());
        assert_eq!(
            id,
            LibId::from_str(
                "alu:650XHPmh-WpXWR5R-Uz4B5jX-jeDqcyr-HXpdZxY-aX9gfO4#plasma-tunnel-mama"
            )
            .unwrap()
        );
        assert_eq!(id, LibId::from_str("alu:650XHPmhWpXWR5RUz4B5jXjeDqcyrHXpdZxYaX9gfO4").unwrap());
        assert_eq!(
            id,
            LibId::from_str("alu:650XHPmhWpXWR5RUz4B5jXjeDqcyrHXpdZxYaX9gfO4#plasma-tunnel-mama")
                .unwrap()
        );

        assert_eq!(id, LibId::from_str("650XHPmhWpXWR5RUz4B5jXjeDqcyrHXpdZxYaX9gfO4").unwrap());
    }
}
