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

//! AluVM instruction set architecture

#[macro_use]
mod asm;
mod bytecode;
mod exec;
mod flags;
mod instr;
pub mod opcodes;

pub use bytecode::{Bytecode, BytecodeError};
pub use exec::{ExecStep, InstructionSet};
pub use flags::{
    DeleteFlag, ExtendFlag, Flag, FloatEqFlag, InsertFlag, IntFlags, MergeFlag, NoneEqFlag,
    ParseFlagError, RoundingFlag, SignFlag, SplitFlag,
};
pub use instr::{
    ArithmeticOp, BitwiseOp, BytesOp, CmpOp, ControlFlowOp, Curve25519Op, DigestOp, Instr, MoveOp,
    PutOp, ReservedOp, Secp256k1Op,
};

/// List of standardised ISA extensions.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[non_exhaustive]
pub enum Isa {
    /// Core ISA instruction set
    #[display("ALU")]
    Alu,

    /// Floating-point operations
    #[display("FLOAT")]
    Float,

    /// Bitcoin-specific cryptographic hash functions
    #[display("BPDIGEST")]
    BpDigest,

    /// Operations on Secp256k1 curve
    #[display("SECP256")]
    Secp256k1,

    /// Operations on Curve25519
    #[display("ED25519")]
    Curve25519,

    /// ALU runtime extensions
    #[display("ALURE")]
    AluRe,

    /// Bitcoin protocol-specific instructions
    #[display("BP")]
    Bp,

    /// RGB-specific instructions
    #[display("RGB")]
    Rgb,

    /// Lightning network protocol-specific instructions
    #[display("LNP")]
    Lnp,

    /// Instructions for SIMD
    #[display("SIMD")]
    Simd,

    /// Instructions for biologically-inspired cognitive architectures
    #[display("REBICA")]
    Rebica,
}

impl Default for Isa {
    #[inline]
    fn default() -> Self { Isa::Alu }
}

impl Isa {
    /// Enumerates all ISA extension variants
    pub const fn all() -> [Isa; 11] {
        [
            Isa::Alu,
            Isa::Float,
            Isa::BpDigest,
            Isa::Secp256k1,
            Isa::Curve25519,
            Isa::AluRe,
            Isa::Bp,
            Isa::Rgb,
            Isa::Lnp,
            Isa::Simd,
            Isa::Rebica,
        ]
    }
}
