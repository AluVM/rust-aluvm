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

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::{self, Display, Formatter};
use core::hash::{Hash as RustHash, Hasher};
use core::marker::PhantomData;
use core::str::FromStr;

use amplify::num::u24;
use bech32::{FromBase32, ToBase32};
use bitcoin_hashes::{sha256, sha256t, Hash, HashEngine};

use super::constants::*;
use super::{Cursor, Read};
use crate::data::ByteStr;
use crate::isa::{Bytecode, BytecodeError, ExecStep, Instr, InstructionSet, ReservedOp};
use crate::libs::CodeEofError;
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
#[wrapper(Debug, LowerHex, Index, IndexRange, IndexFrom, IndexTo, IndexFull)]
pub struct LibId(sha256t::Hash<LibIdTag>);

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
pub struct Lib<E = ReservedOp>
where
    E: InstructionSet,
{
    libs_segment: LibSeg,
    code_segment: ByteStr,
    data_segment: Box<[u8]>,
    instruction_set: PhantomData<E>,
}

impl<E> Display for Lib<E>
where
    E: InstructionSet,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("ISAE: ")?;
        f.write_str(&Instr::<E>::isa_string())?;
        f.write_str("\nCODE: ")?;
        Display::fmt(&self.code_segment, f)?;
        f.write_str("\nDATA: ")?;
        let data = ByteStr::with(&self.data_segment);
        Display::fmt(&data, f)?;
        f.write_str("\nLIBS: ")?;
        Display::fmt(&self.libs_segment, f)
    }
}

impl<E> PartialEq for Lib<E>
where
    E: InstructionSet,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.id().eq(&other.id()) }
}

impl<E> Eq for Lib<E> where E: InstructionSet {}

impl<E> PartialOrd for Lib<E>
where
    E: InstructionSet,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl<E> Ord for Lib<E>
where
    E: InstructionSet,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering { self.id().cmp(&other.id()) }
}

impl<E> RustHash for Lib<E>
where
    E: InstructionSet,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) { state.write(&self.id()[..]) }
}

/// Errors while processing binary-encoded segment data
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum SegmentError {
    /// the size of the CODE segment is {0}, which exceeds [`CODE_SEGMENT_MAX_LEN`]
    CodeSegmentTooLarge(usize),

    /// the size of the DATA segment is {0}, which exceeds [`DATA_SEGMENT_MAX_LEN`]
    DataSegmentTooLarge(usize),

    /// the size of ISAE (instruction set extensions) segment is {0}, which exceeds
    /// [`ISAE_SEGMENT_MAX_LEN`]
    IsaSegmentTooLarge(usize),

    /// number of ISA ids in ISAE segment is {0}, which exceeds [`ISAE_SEGMENT_MAX_COUNT`]
    IsaSegmentTooManyExt(usize),

    /// ISA id {0} has a wrong length outside of [`ISA_ID_MIN_LEN`]`..=`[`ISA_ID_MAX_LEN`] bounds
    IsaIdWrongLength(String),

    /// ISA id {0} includes wrong symbols (must contain only uppercase alphanumeric and start with
    /// letter)
    IsaIdWrongSymbols(String),

    /// ISA id {0} has a wrong length
    IsaNotSupported(String),
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

impl<E> Lib<E>
where
    E: InstructionSet,
{
    /// Constructs library from raw data split into segments
    pub fn with(
        isa: &str,
        bytecode: Vec<u8>,
        data: Vec<u8>,
        libs: LibSeg,
    ) -> Result<Lib<E>, SegmentError> {
        if isa.len() > ISAE_SEGMENT_MAX_LEN {
            return Err(SegmentError::IsaSegmentTooLarge(isa.len()));
        }
        let isa_codes: Vec<_> = isa.split(' ').collect();
        if isa_codes.len() > ISAE_SEGMENT_MAX_COUNT {
            return Err(SegmentError::IsaSegmentTooManyExt(isa_codes.len()));
        }
        for isae in isa_codes {
            if !(ISA_ID_MIN_LEN..=ISA_ID_MAX_LEN).contains(&isae.len()) {
                return Err(SegmentError::IsaIdWrongLength(isae.to_owned()));
            }
            if isae.chars().any(|ch| !ISA_ID_ALLOWED_CHARS.contains(&ch))
                || isae
                    .chars()
                    .next()
                    .map(|ch| !ISA_ID_ALLOWED_FIRST_CHAR.contains(&ch))
                    .unwrap_or_default()
            {
                return Err(SegmentError::IsaIdWrongSymbols(isae.to_owned()));
            }
            if !E::is_supported(isae) {
                return Err(SegmentError::IsaNotSupported(isae.to_owned()));
            }
        }

        if bytecode.len() > CODE_SEGMENT_MAX_LEN {
            return Err(SegmentError::CodeSegmentTooLarge(bytecode.len()));
        }
        if data.len() > DATA_SEGMENT_MAX_LEN {
            return Err(SegmentError::DataSegmentTooLarge(data.len()));
        }
        Ok(Self {
            libs_segment: libs,
            code_segment: ByteStr::with(bytecode),
            data_segment: Box::from(data),
            instruction_set: Default::default(),
        })
    }

    /// Assembles library from the provided instructions by encoding them into bytecode
    pub fn assemble<Isae>(code: &[Isae]) -> Result<Lib<E>, AssemblerError>
    where
        Isae: InstructionSet,
    {
        let call_sites = code.iter().filter_map(|instr| instr.call_site());
        let libs_segment = LibSeg::from(call_sites)?;

        let mut code_segment = ByteStr::default();
        let mut writer = Cursor::new(&mut code_segment.bytes[..], &libs_segment);
        for instr in code.iter() {
            instr.write(&mut writer)?;
        }
        let pos = writer.pos();
        let data = writer.into_data_segment();
        code_segment.adjust_len(pos, false);

        Ok(Lib {
            libs_segment,
            code_segment,
            data_segment: Box::from(data),
            instruction_set: PhantomData::<E>::default(),
        })
    }

    /// Disassembles library into a set of instructions
    pub fn disassemble(&self) -> Result<Vec<Instr<E>>, CodeEofError> {
        let mut code = Vec::new();
        let mut reader = Cursor::with(&self.code_segment, &*self.data_segment, &self.libs_segment);
        while !reader.is_end() {
            code.push(Instr::read(&mut reader)?);
        }
        Ok(code)
    }

    /// Returns hash identifier [`LibId`], representing the library in a unique way.
    ///
    /// Lib ID is computed as SHA256 tagged hash of the serialized library segments (ISAE, code,
    /// data).
    #[inline]
    pub fn id(&self) -> LibId {
        let mut engine = LibId::engine();
        let isae = &*self.isae_segment();
        let code = &self.code_segment();
        let data = &self.data_segment();
        let libs = &self.libs_segment();
        engine.input(&(isae.len() as u8).to_le_bytes()[..]);
        engine.input(isae);
        engine.input(&(code.len() as u16).to_le_bytes()[..]);
        engine.input(code);
        engine.input(&u24::with(data.len() as u32).to_le_bytes()[..]);
        engine.input(data);
        engine.input(&(libs.set.len() as u16).to_le_bytes()[..]);
        libs.set.iter().for_each(|lib| engine.input(&lib[..]));
        LibId::from_engine(engine)
    }

    /// Returns ISA data
    #[inline]
    pub fn isae_segment(&self) -> Box<[u8]> { Instr::<E>::isa_id() }

    /// Returns reference to code segment
    #[inline]
    pub fn code_segment(&self) -> &[u8] { self.code_segment.as_ref() }

    /// Returns reference to data segment
    #[inline]
    pub fn data_segment(&self) -> &[u8] { self.data_segment.as_ref() }

    /// Returns reference to libraries segment
    #[inline]
    pub fn libs_segment(&self) -> &LibSeg { &self.libs_segment }

    /// Executes library code starting at entrypoint
    ///
    /// # Returns
    ///
    /// Location for the external code jump, if any
    pub fn run(&self, entrypoint: u16, registers: &mut CoreRegs) -> Option<LibSite> {
        let mut cursor =
            Cursor::with(&self.code_segment.bytes[..], &*self.data_segment, &self.libs_segment);
        let lib_hash = self.id();
        cursor.seek(entrypoint);

        while !cursor.is_eof() {
            #[cfg(all(debug_assertions, feature = "std"))]
            let pos = cursor.pos();

            let instr = Instr::<E>::read(&mut cursor).ok()?;
            let next = instr.exec(registers, LibSite::with(cursor.pos(), lib_hash));

            #[cfg(all(debug_assertions, feature = "std"))]
            eprint!("\n@{:06}> {:48}; st0={}", pos, instr.to_string(), registers.st0);

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
                    cursor.seek(pos)
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

/// Unable to add a library to the library segment: maximum number of libraries (2^16) exceeded
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
        let set = source.into_iter().map(|site| site.lib).collect::<BTreeSet<LibId>>();
        if set.len() > LIBS_SEGMENT_MAX_COUNT {
            return Err(LibSegOverflow);
        }
        let table = set.iter().enumerate().map(|(index, id)| (index as u8, *id)).collect();
        Ok(LibSeg { set, table })
    }

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
    pub fn index(&self, lib: LibId) -> Option<u16> {
        self.set.iter().position(|l| *l == lib).map(|i| i as u16)
    }
}

impl Display for LibSeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.set.iter().try_for_each(|lib| {
            Display::fmt(lib, f)?;
            f.write_str("\n      ")
        })
    }
}
