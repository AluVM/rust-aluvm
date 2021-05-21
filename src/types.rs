// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[cfg(feature = "std")]
use std::fmt::{self, Display, Formatter};
#[cfg(feature = "std")]
use std::str::FromStr;

use amplify::num::{u1024, u256, u512};
use bitcoin_hashes::Hash;

use crate::instr::encoding::{compile, Cursor, EncodeError};
use crate::instr::Bytecode;
use crate::InstructionSet;

const LIB_HASH_MIDSTATE: [u8; 32] = [
    156, 224, 228, 230, 124, 17, 108, 57, 56, 179, 202, 242, 195, 15, 80, 137,
    211, 243, 147, 108, 71, 99, 110, 96, 125, 179, 62, 234, 221, 198, 240, 201,
];

sha256t_hash_newtype!(
    LibHash,
    LibHashTag,
    LIB_HASH_MIDSTATE,
    64,
    doc = "Library reference: a hash of the library code",
    false
);

/// AluVM executable code library
#[cfg_attr(
    feature = "std",
    derive(Debug, Display),
    display("{bytecode}", alt = "{bytecode:#}")
)]
pub struct Lib {
    bytecode: Blob,
    pub cursor: Cursor<[u8; u16::MAX as usize]>,
}

impl Lib {
    pub fn with<E, I>(code: I) -> Result<Lib, EncodeError>
    where
        E: InstructionSet,
        I: IntoIterator,
        <I as IntoIterator>::Item: InstructionSet,
    {
        let bytecode = compile::<E, _>(code)?;
        let cursor = Cursor::with(bytecode.bytes);
        Ok(Lib { bytecode, cursor })
    }

    /// Returns hash identifier [`LibHash`], representing the library in a
    /// unique way.
    ///
    /// Lib hash is computed as SHA256 tagged hash of the serialized library
    /// bytecode.
    pub fn lib_hash(&self) -> LibHash {
        LibHash::hash(&self.bytecode.bytes)
    }

    /// Calculates length of bytecode encoding in bytes
    pub fn byte_count(&self) -> u16 {
        self.bytecode.len
    }

    /// Returns bytecode reference
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode.as_ref()
    }
}

impl AsRef<[u8]> for Lib {
    fn as_ref(&self) -> &[u8] {
        self.bytecode.as_ref()
    }
}

/// Location within a library
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[cfg_attr(feature = "std", derive(Display), display("{pos:#06X}@{lib}"))]
pub struct LibSite {
    /// Library hash
    pub lib: LibHash,

    /// Offset from the beginning of the code, in bytes
    pub pos: u16,
}

impl LibSite {
    /// Constricts library site reference from a given position and library hash
    /// value
    pub fn with(pos: u16, lib: LibHash) -> LibSite {
        LibSite { lib, pos }
    }
}

/// Large binary bytestring object
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Blob {
    /// Slice length
    pub len: u16,

    /// Slice bytes
    pub bytes: [u8; u16::MAX as usize],
}

impl Default for Blob {
    fn default() -> Blob {
        Blob {
            len: 0,
            bytes: [0u8; u16::MAX as usize],
        }
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

#[cfg(feature = "std")]
impl Display for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        let vec = Vec::from(&self.bytes[..self.len as usize]);
        if let Ok(s) = String::from_utf8(vec) {
            f.write_str("\"")?;
            f.write_str(&s)?;
            f.write_str("\"")
        } else if f.alternate() && self.len > 4 {
            write!(
                f,
                "{}..{}",
                self.bytes[..4].to_hex(),
                self.bytes[(self.len as usize - 4)..].to_hex()
            )
        } else {
            f.write_str(&self.bytes[0usize..(self.len as usize)].to_hex())
        }
    }
}

/// Copy'able variable length slice
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Value {
    /// Slice length
    pub len: u16,

    /// Slice bytes
    pub bytes: [u8; 1024],
}

impl Default for Value {
    fn default() -> Value {
        Value {
            len: 0,
            bytes: [0u8; 1024],
        }
    }
}

impl Value {
    /// Constructs value from slice of bytes.
    ///
    /// Panics if the length of the slice is greater than 1024 bytes.
    pub fn with(slice: impl AsRef<[u8]>) -> Value {
        let len = slice.as_ref().len();
        let mut bytes = [0u8; 1024];
        bytes[0..len].copy_from_slice(slice.as_ref());
        Value {
            len: len as u16,
            bytes,
        }
    }

    /// Constructs value from hex string
    #[cfg(feature = "std")]
    pub fn from_hex(s: &str) -> Result<Value, amplify::hex::Error> {
        use amplify::hex::FromHex;
        let s = s.trim_start_matches("0x");
        let len = s.len() / 2;
        if len > 1024 {
            return Err(amplify::hex::Error::InvalidLength(1024, len));
        }
        let mut bytes = [0u8; 1024];
        let hex = Vec::<u8>::from_hex(&s)?;
        bytes[0..len].copy_from_slice(&hex);
        Ok(Value {
            len: hex.len() as u16,
            bytes,
        })
    }

    /// Serializes value in hexadecimal format to a string
    #[cfg(feature = "std")]
    pub fn to_hex(&self) -> String {
        use std::fmt::Write;
        let mut ret = String::with_capacity(2usize * self.len as usize + 2);
        write!(ret, "0x");
        for ch in &self.bytes {
            write!(ret, "{:02x}", ch).expect("writing to string");
        }
        ret
    }
}

/// Errors parsing literal values in AluVM assembly code
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg(feature = "std")]
#[derive(Display, Error, From)]
#[display(inner)]
pub enum LiteralParseError {
    /// Error parsing hexadecimal literal
    #[from]
    Hex(amplify::hex::Error),

    /// Error parsing decimal literal
    #[from]
    Int(std::num::ParseIntError),

    /// Unknown literal
    #[display("unknown token `{0}` while parsing AluVM assembly literal")]
    UnknownLiteral(String),
}

#[cfg(feature = "std")]
impl FromStr for Value {
    type Err = LiteralParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            Value::from_hex(s).map_err(LiteralParseError::from)
        } else if s.starts_with("-") {
            // TODO: use arbitrary-precision type `FromStr`
            Ok(Value::from(i128::from_str(s)?))
        } else {
            // TODO: use arbitrary-precision type `FromStr`
            let val = u128::from_str(s)?;
            Ok(match val {
                0..=0xFF => Value::from(val as u8),
                0..=0xFFFF => Value::from(val as u16),
                0..=0xFFFFFFFF => Value::from(val as u32),
                0..=0xFFFFFFFFFFFFFFFF => Value::from(val as u64),
                _ => Value::from(val),
            })
        }
    }
}

#[cfg(feature = "std")]
impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        f.write_str("0x")?;
        if f.alternate() && self.len > 4 {
            write!(
                f,
                "{}..{}",
                self.bytes[..4].to_hex(),
                self.bytes[(self.len as usize - 4)..].to_hex()
            )
        } else {
            f.write_str(&self.bytes[0usize..(self.len as usize)].to_hex())
        }
    }
}

macro_rules! impl_value_bytes_conv {
    ($len:literal) => {
        impl From<Value> for [u8; $len] {
            fn from(mut val: Value) -> Self {
                let mut bytes = [0u8; $len];
                let clean = Value::default();
                val.bytes[$len..].copy_from_slice(&clean.bytes[$len..]);
                bytes.copy_from_slice(&val.bytes[0..$len]);
                bytes
            }
        }

        impl From<[u8; $len]> for Value {
            fn from(val: [u8; $len]) -> Value {
                let mut bytes = [0u8; 1024];
                bytes[0..$len].copy_from_slice(&val[..]);
                Value { len: $len, bytes }
            }
        }
    };
}

macro_rules! impl_value_ty_conv {
    ($ty:ident, $len:literal) => {
        impl From<Value> for $ty {
            fn from(val: Value) -> Self {
                $ty::from_le_bytes(<[u8; $len]>::from(val))
            }
        }

        impl From<$ty> for Value {
            fn from(val: $ty) -> Self {
                Value::from(&val)
            }
        }
        impl From<&$ty> for Value {
            fn from(val: &$ty) -> Self {
                let mut bytes = [0u8; 1024];
                let le = val.to_le_bytes();
                bytes[0..le.len()].copy_from_slice(&le[..]);
                Value {
                    len: le.len() as u16,
                    bytes,
                }
            }
        }
    };
}

impl_value_bytes_conv!(1);
impl_value_bytes_conv!(2);
impl_value_bytes_conv!(4);
impl_value_bytes_conv!(8);
impl_value_bytes_conv!(16);
impl_value_bytes_conv!(20);
impl_value_bytes_conv!(32);
impl_value_bytes_conv!(64);
impl_value_bytes_conv!(128);
impl_value_bytes_conv!(256);
impl_value_bytes_conv!(512);
impl_value_bytes_conv!(1024);

impl_value_ty_conv!(u8, 1);
impl_value_ty_conv!(u16, 2);
impl_value_ty_conv!(u32, 4);
impl_value_ty_conv!(u64, 8);
impl_value_ty_conv!(u128, 16);
impl_value_ty_conv!(u256, 32);
impl_value_ty_conv!(u512, 64);
impl_value_ty_conv!(u1024, 128);

impl_value_ty_conv!(i8, 1);
impl_value_ty_conv!(i16, 2);
impl_value_ty_conv!(i32, 4);
impl_value_ty_conv!(i64, 8);
impl_value_ty_conv!(i128, 16);
