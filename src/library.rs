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
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};
use core::marker::PhantomData;

use bitcoin_hashes::{sha256, Hash};

use crate::instr::serialize::{compile, Bytecode, EncodeError};
use crate::instr::{ExecStep, NOp};
use crate::{ByteStr, Cursor, Instr, InstructionSet, Registers};

const LIB_HASH_MIDSTATE: [u8; 32] = [
    156, 224, 228, 230, 124, 17, 108, 57, 56, 179, 202, 242, 195, 15, 80, 137, 211, 243, 147, 108,
    71, 99, 110, 96, 125, 179, 62, 234, 221, 198, 240, 201,
];

pub const DATA_SEGMENT_LIMIT: usize = 1 << 24;

sha256t_hash_newtype!(
    LibHash,
    LibHashTag,
    LIB_HASH_MIDSTATE,
    64,
    doc = "Library reference: a hash of the library code",
    false
);

/// AluVM executable code library
#[derive(Debug)]
pub struct Lib<E = NOp>
where
    E: InstructionSet,
{
    code_segment: ByteStr,
    data_segment: Box<[u8]>,
    instruction_set: PhantomData<E>,
}

impl<E> Display for Lib<E>
where
    E: InstructionSet,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&E::ids().into_iter().collect::<Vec<_>>().join("+"))?;
        f.write_str(":")?;
        Display::fmt(&self.code_segment, f)?;
        f.write_str(":")?;
        write!(f, "#{}", sha256::Hash::hash(&self.data_segment))
    }
}

impl<E> Lib<E>
where
    E: InstructionSet,
{
    /// Constructs library for the provided instructions by encoding them into bytecode
    pub fn with<I>(code: I, data: Option<&[u8]>) -> Result<Lib<E>, EncodeError>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: InstructionSet,
    {
        let code_segment = compile::<E, _>(code)?;
        let len = data.map(<[u8]>::len).unwrap_or_default();
        if len >= DATA_SEGMENT_LIMIT {
            return Err(EncodeError::DataSegmentTooLarge(len));
        }
        let data_segment = data.map(Box::from).unwrap_or_default();
        Ok(Lib { code_segment, data_segment, instruction_set: PhantomData::<E>::default() })
    }

    /// Returns hash identifier [`LibHash`], representing the library in a unique way.
    ///
    /// Lib hash is computed as SHA256 tagged hash of the serialized library bytecode.
    pub fn lib_hash(&self) -> LibHash { LibHash::hash(&*self.code_segment.bytes) }

    /// Calculates length of bytecode encoding in bytes
    pub fn byte_count(&self) -> usize { self.code_segment.len() }

    /// Returns bytecode reference
    pub fn bytecode(&self) -> &[u8] { self.code_segment.as_ref() }

    /// Executes library code starting at entrypoint
    pub fn run(&self, entrypoint: u16, registers: &mut Registers) -> Option<LibSite> {
        let mut cursor = Cursor::with(&self.code_segment.bytes[..]);
        let lib_hash = self.lib_hash();
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

impl<E> AsRef<[u8]> for Lib<E>
where
    E: InstructionSet,
{
    fn as_ref(&self) -> &[u8] { self.code_segment.as_ref() }
}

/// Location within a library
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Display)]
#[display("{pos:#06X}@{lib}")]
pub struct LibSite {
    /// Library hash
    pub lib: LibHash,

    /// Offset from the beginning of the code, in bytes
    pub pos: u16,
}

impl LibSite {
    /// Constricts library site reference from a given position and library hash
    /// value
    pub fn with(pos: u16, lib: LibHash) -> LibSite { LibSite { lib, pos } }
}
