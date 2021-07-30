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

//! Constants defined for AluVM libraries

#![allow(missing_docs)]

pub const CODE_SEGMENT_MAX_LEN: usize = 1 << 16;

pub const DATA_SEGMENT_MAX_LEN: usize = 1 << 16;

/// Maximum number of libraries that may be referenced (used by) any other librari; i.e. limit for
/// the number of records inside libs segment.
pub const LIBS_SEGMENT_MAX_COUNT: usize = 1 << 8;

/// Maximum total number of libraries which may be used by a single program; i.e. maximal number of
/// nodes in a library dependency tree.
pub const LIBS_MAX_TOTAL: usize = 1024;

pub const ISAE_SEGMENT_MAX_LEN: usize = 0xFF;

pub const ISAE_SEGMENT_MAX_COUNT: usize = 32;

pub const ISA_ID_MIN_LEN: usize = 2;

pub const ISA_ID_MAX_LEN: usize = 8;

pub const ISA_ID_ALLOWED_CHARS: [char; 36] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
pub const ISA_ID_ALLOWED_FIRST_CHAR: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub const ISA_ID_ALU: &str = "ALU";
pub const ISA_ID_BPDIGEST: &str = "BPDIGEST";
pub const ISA_ID_SECP256K: &str = "SECP256K";
pub const ISA_ID_ED25519: &str = "ED25519";

pub const ISA_ID_ALURE: &str = "ALURE";
pub const ISA_ID_SIMD: &str = "SIMD";
pub const ISA_ID_INET2: &str = "INET4";
pub const ISA_ID_WEB4: &str = "WEB4";

pub const ISA_ID_BITCOIN: &str = "BITCOIN";
pub const ISA_ID_BP: &str = "BP";
pub const ISA_ID_RGB: &str = "RGB";
pub const ISA_ID_LNP: &str = "LNP";

pub const ISA_ID_REBICA: &str = "REBICA";
