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
use std::fmt::{self, Display, Formatter, LowerHex, UpperHex};

use amplify::num::{u1024, u256, u512};

/// Library reference: a hash of the library code
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[cfg_attr(feature = "std", derive(Display), display(LowerHex))]
#[derive(Wrapper, From)]
pub struct LibHash([u8; 32]);

/// Location within a library
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[cfg_attr(feature = "std", derive(Display), display("{pos:#06X}@{lib}"))]
pub struct LibSite {
    /// Library hash
    lib: LibHash,

    /// Offset from the beginning of the code, in bytes
    pos: u16,
}

#[cfg(feature = "std")]
impl LowerHex for LibHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        if f.alternate() {
            write!(
                f,
                "{}..{}",
                self.0[..4].to_hex(),
                self.0[(self.0.len() - 4)..].to_hex()
            )
        } else {
            f.write_str(&self.0.to_hex())
        }
    }
}

#[cfg(feature = "std")]
impl UpperHex for LibHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
        if f.alternate() {
            write!(
                f,
                "{}..{}",
                self.0[..4].to_hex().to_ascii_uppercase(),
                self.0[(self.0.len() - 4)..].to_hex().to_ascii_uppercase()
            )
        } else {
            f.write_str(&self.0.to_hex().to_ascii_uppercase())
        }
    }
}

/// Copy'able variable length slice
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Blob {
    /// Slice length
    pub len: u16,

    /// Slice bytes
    pub bytes: [u8; 1024],
}

impl Default for Blob {
    fn default() -> Blob {
        Blob {
            len: 0,
            bytes: [0u8; 1024],
        }
    }
}

#[cfg(feature = "std")]
impl Display for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::hex::ToHex;
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

macro_rules! impl_blob_bytes_conv {
    ($len:literal) => {
        impl From<Blob> for [u8; $len] {
            fn from(mut val: Blob) -> Self {
                let mut bytes = [0u8; $len];
                let clean = Blob::default();
                val.bytes[$len..].copy_from_slice(&clean.bytes[$len..]);
                bytes.copy_from_slice(&val.bytes[0..$len]);
                bytes
            }
        }

        impl From<[u8; $len]> for Blob {
            fn from(val: [u8; $len]) -> Blob {
                let mut bytes = [0u8; 1024];
                bytes[0..$len].copy_from_slice(&val[..]);
                Blob { len: $len, bytes }
            }
        }
    };
}

macro_rules! impl_blob_ty_conv {
    ($ty:ident, $len:literal) => {
        impl From<Blob> for $ty {
            fn from(val: Blob) -> Self {
                $ty::from_le_bytes(<[u8; $len]>::from(val))
            }
        }

        impl From<$ty> for Blob {
            fn from(val: $ty) -> Self {
                Blob::from(&val)
            }
        }
        impl From<&$ty> for Blob {
            fn from(val: &$ty) -> Self {
                let mut bytes = [0u8; 1024];
                let le = val.to_le_bytes();
                bytes[0..le.len()].copy_from_slice(&le[..]);
                Blob {
                    len: le.len() as u16,
                    bytes,
                }
            }
        }
    };
}

impl_blob_bytes_conv!(1);
impl_blob_bytes_conv!(2);
impl_blob_bytes_conv!(4);
impl_blob_bytes_conv!(8);
impl_blob_bytes_conv!(16);
impl_blob_bytes_conv!(20);
impl_blob_bytes_conv!(32);
impl_blob_bytes_conv!(64);
impl_blob_bytes_conv!(128);
impl_blob_bytes_conv!(256);
impl_blob_bytes_conv!(512);
impl_blob_bytes_conv!(1024);

impl_blob_ty_conv!(u8, 1);
impl_blob_ty_conv!(u16, 2);
impl_blob_ty_conv!(u32, 4);
impl_blob_ty_conv!(u64, 8);
impl_blob_ty_conv!(u128, 16);
impl_blob_ty_conv!(u256, 32);
impl_blob_ty_conv!(u512, 64);
impl_blob_ty_conv!(u1024, 128);

impl_blob_ty_conv!(i8, 1);
impl_blob_ty_conv!(i16, 2);
impl_blob_ty_conv!(i32, 4);
impl_blob_ty_conv!(i64, 8);
impl_blob_ty_conv!(i128, 16);
