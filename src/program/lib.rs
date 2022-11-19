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

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::hash::{Hash as RustHash, Hasher};
use core::str::FromStr;

use bech32::{FromBase32, ToBase32};
use bitcoin_hashes::{sha256, sha256t, Hash, HashEngine};

use super::{Cursor, Read};
use crate::data::ByteStr;
use crate::isa::{BytecodeError, ExecStep, InstructionSet};
use crate::program::segs::IsaSeg;
use crate::program::{CodeEofError, LibSeg, LibSegOverflow, SegmentError};
use crate::reg::CoreRegs;

const LIB_ID_MIDSTATE: [u8; 32] = [
    156, 224, 228, 230, 124, 17, 108, 57, 56, 179, 202, 242, 195, 15, 80, 137, 211, 243, 147, 108,
    71, 99, 110, 96, 125, 179, 62, 234, 221, 198, 240, 201,
];

/// Bech32m prefix for library id encoding
pub const LIBID_BECH32_HRP: &str = "alu";

/// Tag used for [`LibId`] hash type
pub struct LibIdTag;

impl sha256t::Tag for LibIdTag {
    #[inline]
    fn engine() -> sha256::HashEngine {
        let midstate = sha256::Midstate::from_inner(LIB_ID_MIDSTATE);
        sha256::HashEngine::from_midstate(midstate, 64)
    }
}

/// A library identifier
///
/// Represents commitment to the library data; any two distinct programs are guaranteed (with SHA256
/// collision resistance level) to have a distinct library ids.
#[derive(Wrapper, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, From)]
#[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[wrapper(Debug, LowerHex, Index, IndexRange, IndexFrom, IndexTo, IndexFull)]
pub struct LibId(sha256t::Hash<LibIdTag>);

impl LibId {
    /// Computes LibId from the provided data
    pub fn with(
        isae: impl AsRef<str>,
        code: impl AsRef<[u8]>,
        data: impl AsRef<[u8]>,
        libs: &LibSeg,
    ) -> LibId {
        let isae = isae.as_ref();
        let code = code.as_ref();
        let data = data.as_ref();
        let mut engine = LibId::engine();
        engine.input(&(isae.len() as u8).to_le_bytes()[..]);
        engine.input(isae.as_bytes());
        engine.input(&code.len().to_le_bytes()[..]);
        engine.input(code.as_ref());
        engine.input(&data.len().to_le_bytes()[..]);
        engine.input(data.as_ref());
        engine.input(&[libs.count()]);
        libs.iter().for_each(|lib| engine.input(&lib[..]));
        LibId::from_engine(engine)
    }

    /// Constructs library id from a binary representation of the hash data
    #[inline]
    pub fn from_bytes(array: [u8; LibId::LEN]) -> LibId {
        LibId(sha256t::Hash::<LibIdTag>::from_inner(array))
    }

    /// Returns fixed-size array of inner representation of the library id
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] { self.0.as_inner() }
}

impl Borrow<[u8]> for LibId {
    #[inline]
    fn borrow(&self) -> &[u8] { self.as_inner() }
}

impl Hash for LibId {
    type Engine = <sha256t::Hash<LibIdTag> as Hash>::Engine;
    type Inner = <sha256t::Hash<LibIdTag> as Hash>::Inner;

    const LEN: usize = 32;
    const DISPLAY_BACKWARD: bool = false;

    #[inline]
    fn engine() -> Self::Engine { sha256t::Hash::<LibIdTag>::engine() }

    #[inline]
    fn from_engine(e: Self::Engine) -> Self { Self(sha256t::Hash::from_engine(e)) }

    #[inline]
    fn from_slice(sl: &[u8]) -> Result<Self, bitcoin_hashes::Error> {
        Ok(Self(sha256t::Hash::from_slice(sl)?))
    }

    #[inline]
    fn into_inner(self) -> Self::Inner { self.0.into_inner() }

    #[inline]
    fn as_inner(&self) -> &Self::Inner { self.0.as_inner() }

    #[inline]
    fn from_inner(inner: Self::Inner) -> Self { Self(sha256t::Hash::from_inner(inner)) }
}

/// Error parsing [`LibId`] bech32m representation
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(doc_comments)]
pub enum LibIdError {
    /// Error reported by bech32 library
    #[display(inner)]
    #[from]
    Bech32(bech32::Error),

    /// LibId must start with `alu1` and not `{0}`
    InvalidHrp(String),

    /// LibId must be encoded with Bech32m variant and not Bech32
    InvalidVariant,

    /// LibId data must have length of 32 bytes
    #[from]
    InvalidLength(bitcoin_hashes::Error),
}

#[cfg(feature = "std")]
impl ::std::error::Error for LibIdError {
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            LibIdError::Bech32(err) => Some(err),
            LibIdError::InvalidLength(err) => Some(err),
            LibIdError::InvalidHrp(_) | LibIdError::InvalidVariant => None,
        }
    }
}

impl Display for LibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes: &[u8] = self.borrow();
        let s = bech32::encode(LIBID_BECH32_HRP, bytes.to_base32(), bech32::Variant::Bech32m)
            .map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

impl FromStr for LibId {
    type Err = LibIdError;

    fn from_str(s: &str) -> Result<Self, LibIdError> {
        let (hrp, b32, variant) = bech32::decode(s)?;
        if hrp != LIBID_BECH32_HRP {
            return Err(LibIdError::InvalidHrp(hrp));
        }
        if variant != bech32::Variant::Bech32m {
            return Err(LibIdError::InvalidVariant);
        }
        let data = Vec::<u8>::from_base32(&b32)?;
        LibId::from_slice(&data).map_err(LibIdError::from)
    }
}

/// AluVM executable code library
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
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
#[cfg_attr(feature = "strict_encoding", derive(StrictEncode, StrictDecode))]
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
