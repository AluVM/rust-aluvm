// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![allow(missing_docs)]
#![allow(clippy::unusual_byte_groupings)]

// Control-flow instructions
pub const INSTR_FAIL: u8 = 0b00_000_000;
pub const INSTR_SUCC: u8 = 0b00_000_001;
pub const INSTR_JMP: u8 = 0b00_000_010;
pub const INSTR_JIF: u8 = 0b00_000_011;
pub const INSTR_ROUTINE: u8 = 0b00_000_100;
pub const INSTR_CALL: u8 = 0b00_000_101;
pub const INSTR_EXEC: u8 = 0b00_000_110;
pub const INSTR_RET: u8 = 0b00_000_111;

// Instructions setting register values
pub const INSTR_ZEROA: u8 = 0b00_001_000;
pub const INSTR_ZEROR: u8 = 0b00_001_001;
pub const INSTR_CLA: u8 = 0b00_001_010;
pub const INSTR_CLR: u8 = 0b00_001_011;
pub const INSTR_PUTA: u8 = 0b00_001_100;
pub const INSTR_PUTR: u8 = 0b00_001_101;
pub const INSTR_PUTIFA: u8 = 0b00_001_110;
pub const INSTR_PUTIFR: u8 = 0b00_001_111;

// Instructions moving and swapping register values
pub const INSTR_MOV: u8 = 0b00_010_000;
pub const INSTR_CPA: u8 = 0b00_010_001;
pub const INSTR_CNA: u8 = 0b00_010_010;
pub const INSTR_CNF: u8 = 0b00_010_011;
pub const INSTR_CPR: u8 = 0b00_010_100;
pub const INSTR_SPY: u8 = 0b00_010_101;
pub const INSTR_CAF: u8 = 0b00_010_110;
pub const INSTR_CFA: u8 = 0b00_010_111;

// Instructions comparing register values
pub const INSTR_LGT: u8 = 0b00_011_000;
pub const INSTR_CMP: u8 = 0b00_011_001;
pub const INSTR_IFZA: u8 = 0b00_011_010;
pub const INSTR_IFZR: u8 = 0b00_011_011;
pub const INSTR_IFNA: u8 = 0b00_011_100;
pub const INSTR_IFNR: u8 = 0b00_011_101;
pub const INSTR_ST: u8 = 0b00_011_110;
pub const INSTR_STINV: u8 = 0b00_011_111;

// Arithmetic instructions
pub const INSTR_ADD: u8 = 0b00_100_000;
pub const INSTR_SUB: u8 = 0b00_100_001;
pub const INSTR_MUL: u8 = 0b00_100_010;
pub const INSTR_DIV: u8 = 0b00_100_011;
pub const INSTR_STP: u8 = 0b00_100_100;
pub const INSTR_NEG: u8 = 0b00_100_101;
pub const INSTR_ABS: u8 = 0b00_100_110;
pub const INSTR_REM: u8 = 0b00_100_111;

// Bit operations & boolean algebra instructions
pub const INSTR_AND: u8 = 0b00_101_000;
pub const INSTR_OR: u8 = 0b00_101_001;
pub const INSTR_XOR: u8 = 0b00_101_010;
pub const INSTR_NOT: u8 = 0b00_101_011;
pub const INSTR_SHF: u8 = 0b00_101_100;
pub const INSTR_SHC: u8 = 0b00_101_101;
pub const INSTR_REVA: u8 = 0b00_101_110;
pub const INSTR_REVR: u8 = 0b00_101_111;

//  Operations on byte strings
pub const INSTR_PUT: u8 = 0b00_110_000;
pub const INSTR_MVS: u8 = 0b00_110_001;
pub const INSTR_SWP: u8 = 0b00_110_010;
pub const INSTR_FILL: u8 = 0b00_110_011;
pub const INSTR_LEN: u8 = 0b00_110_100;
pub const INSTR_CNT: u8 = 0b00_110_101;
pub const INSTR_EQ: u8 = 0b00_110_110;
pub const INSTR_CON: u8 = 0b00_110_111;

pub const INSTR_FIND: u8 = 0b00_111_000;
pub const INSTR_EXTR: u8 = 0b00_111_001;
pub const INSTR_INJ: u8 = 0b00_111_010;
pub const INSTR_JOIN: u8 = 0b00_111_011;
pub const INSTR_SPLT: u8 = 0b00_111_100;
pub const INSTR_INS: u8 = 0b00_111_101;
pub const INSTR_DEL: u8 = 0b00_111_110;
pub const INSTR_REV: u8 = 0b00_111_111;

// Cryptographic hashing functions
pub const INSTR_RIPEMD: u8 = 0b01_000_000;
pub const INSTR_SHA256: u8 = 0b01_000_001;
pub const INSTR_SHA512: u8 = 0b01_000_010;
pub const INSTR_HASH1: u8 = 0b01_000_011; // Reserved for future use
pub const INSTR_HASH2: u8 = 0b01_000_100; // Reserved for future use
pub const INSTR_HASH3: u8 = 0b01_000_101; // Reserved for future use
pub const INSTR_HASH4: u8 = 0b01_000_110; // Reserved for future use
pub const INSTR_HASH5: u8 = 0b01_000_111; // Reserved for future use

// Operations on Secp256k1 elliptic curve
pub const INSTR_SECP_GEN: u8 = 0b01_001_000;
pub const INSTR_SECP_MUL: u8 = 0b01_001_001;
pub const INSTR_SECP_ADD: u8 = 0b01_001_010;
pub const INSTR_SECP_NEG: u8 = 0b01_001_011;
// Operations on Curve25519 elliptic curve
pub const INSTR_ED_GEN: u8 = 0b01_001_100;
pub const INSTR_ED_MUL: u8 = 0b01_001_101;
pub const INSTR_ED_ADD: u8 = 0b01_001_110;
pub const INSTR_ED_NEG: u8 = 0b01_001_111;

// Reserved operations which can be provided by a host environment
pub const INSTR_EXT_FROM: u8 = 0b10_000_000;
pub const INSTR_EXT_TO: u8 = 0b10_111_111;
// Reserved for future use
pub const INSTR_RESV_FROM: u8 = 0b11_000_000;
pub const INSTR_RESV_TO: u8 = 0b11_111_110;

// No-operation instruction
pub const INSTR_NOP: u8 = 0b11_111_111;
