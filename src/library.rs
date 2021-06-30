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

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};
use core::marker::PhantomData;
use core::ops::Index;

use amplify_num::u24;
use bitcoin_hashes::{Hash, HashEngine};

use crate::bytecoder::Read;
use crate::instr::serialize::{Bytecode, DecodeError, EncodeError};
use crate::instr::{ExecStep, NOp};
use crate::{ByteStr, Cursor, Instr, InstructionSet, Registers};

const LIB_ID_MIDSTATE: [u8; 32] = [
    156, 224, 228, 230, 124, 17, 108, 57, 56, 179, 202, 242, 195, 15, 80, 137, 211, 243, 147, 108,
    71, 99, 110, 96, 125, 179, 62, 234, 221, 198, 240, 201,
];

sha256t_hash_newtype!(
    LibId,
    LibIdTag,
    LIB_ID_MIDSTATE,
    64,
    doc = "Library reference: a hash of the library code",
    false
);

/// Errors happening during library creation from bytecode & data
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(doc_comments)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum Error {
    /// The size of the code segment exceeds 2^16
    CodeSegmentTooLarge(usize),

    /// The size of the data segment exceeds 2^24
    DataSegmentTooLarge(usize),
}

/// AluVM executable code library
#[derive(Debug, Default)]
pub struct Lib<E = NOp>
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

impl<E> Lib<E>
where
    E: InstructionSet,
{
    /// Constructs library from segments
    pub fn with(bytecode: Vec<u8>, data: Vec<u8>, libs: LibSeg) -> Result<Lib<E>, Error> {
        if bytecode.len() > u16::MAX as usize {
            return Err(Error::CodeSegmentTooLarge(bytecode.len()));
        }
        if data.len() > u24::MAX.as_u32() as usize {
            return Err(Error::DataSegmentTooLarge(data.len()));
        }
        Ok(Self {
            libs_segment: libs,
            code_segment: ByteStr::with(bytecode),
            data_segment: Box::from(data),
            instruction_set: Default::default(),
        })
    }

    /// Assembles library from the provided instructions by encoding them into bytecode
    pub fn assemble<I>(code: I) -> Result<Lib<E>, EncodeError>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: InstructionSet,
    {
        let mut code_segment = ByteStr::default();
        let mut writer = Cursor::new(&mut code_segment.bytes[..]);
        for instr in code.into_iter() {
            instr.write(&mut writer)?;
        }
        let pos = writer.pos();
        let (data, libs_segment) = writer.into_segments();
        code_segment.adjust_len(pos, false);

        Ok(Lib {
            libs_segment,
            code_segment,
            data_segment: Box::from(data),
            instruction_set: PhantomData::<E>::default(),
        })
    }

    /// Disassembles library into a set of instructions
    pub fn disassemble(&self) -> Result<Vec<Instr<E>>, DecodeError> {
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
    pub fn run(&self, entrypoint: u16, registers: &mut Registers) -> Option<LibSite> {
        let mut cursor =
            Cursor::with(&self.code_segment.bytes[..], &*self.data_segment, &self.libs_segment);
        let lib_hash = self.id();
        cursor.seek(entrypoint);

        while !cursor.is_eof() {
            let instr = Instr::<E>::read(&mut cursor).ok()?;
            match instr.exec(registers, LibSite::with(cursor.pos(), lib_hash)) {
                ExecStep::Stop => return None,
                ExecStep::Next => continue,
                ExecStep::Jump(pos) => cursor.seek(pos),
                ExecStep::Call(site) => return Some(site),
            }
        }

        None
    }
}

/// Unable to add a library to the library segment: maximum number of libraries (2^16) exceeded
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Display, Error)]
#[display(doc_comments)]
pub struct LibSegOverflow;

/// Location within a library
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Display)]
#[display("\"{lib}\",{pos:#06X}")]
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

mod private {
    pub trait Sealed {}
    impl Sealed for super::LibSeg {}
    impl Sealed for &super::LibSeg {}
}

#[doc(hidden)]
pub trait LibSegment: private::Sealed {
    /// Returns library id with a given index
    fn lib_at(&self, index: u16) -> Option<LibId>;

    /// Returns index of a library
    fn lib_index(&self, lib: LibId) -> Option<u16>;

    /// Adds library to the collection
    fn insert(&mut self, lib: LibId) -> Result<u16, LibSegOverflow>;
}

/// Library segment data
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct LibSeg {
    set: BTreeSet<LibId>,
    table: BTreeMap<u16, LibId>,
}

impl LibSegment for &LibSeg {
    #[inline]
    fn lib_at(&self, index: u16) -> Option<LibId> { self.table.get(&index).copied() }

    #[inline]
    fn lib_index(&self, lib: LibId) -> Option<u16> {
        self.set.iter().position(|l| *l == lib).map(|i| i as u16)
    }

    fn insert(&mut self, _: LibId) -> Result<u16, LibSegOverflow> {
        panic!("attempt to modify read-only LibSegment")
    }
}

impl LibSegment for LibSeg {
    #[inline]
    fn lib_at(&self, index: u16) -> Option<LibId> { (&self).lib_at(index) }

    #[inline]
    fn lib_index(&self, lib: LibId) -> Option<u16> { (&self).lib_index(lib) }

    fn insert(&mut self, lib: LibId) -> Result<u16, LibSegOverflow> {
        if self.set.len() >= u16::MAX as usize {
            return Err(LibSegOverflow);
        }
        self.set.insert(lib);
        let index = self.lib_index(lib).expect("BTreeSet is broken");
        self.table.insert(index, lib);
        Ok(index)
    }
}

impl Index<u16> for LibSeg {
    type Output = LibId;

    #[inline]
    fn index(&self, index: u16) -> &Self::Output {
        self.table.get(&index).expect("Unknown library")
    }
}

impl Display for LibSeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.set.iter().try_for_each(|lib| write!(f, "#{}", lib))
    }
}
