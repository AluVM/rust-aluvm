// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_use]
mod asm;
pub mod encoding;
mod exec;
mod instr;
mod op_codes;
mod op_types;

pub use encoding::Bytecode;
pub use exec::{ExecStep, InstructionSet};
pub use instr::*;
pub use op_codes::{
    ArithmeticOp, BitwiseOp, BytesOp, CmpOp, ControlFlowOp, Curve25519Op,
    DigestOp, Instr, MoveOp, NOp, PutOp, SecpOp,
};
pub use op_types::{Arithmetics, IncDec, NumType};
