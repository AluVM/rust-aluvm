// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![allow(clippy::branches_sharing_code)]

use amplify_num::u4;

use crate::instr::{Arithmetics, IncDec, NumType};
use crate::reg::{Reg32, Reg8, RegA, RegBlock, RegR, Value};
use crate::{Blob, InstructionSet, LibSite, Reg16};

/// Default instruction extension which treats any operation as NOP
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display("nop")]
pub enum NOp {
    NOp,
}

/// Full set of instructions
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum Instr<Extension = NOp>
where
    Extension: InstructionSet,
{
    /// Control-flow instructions
    // 0b00_000_***
    ControlFlow(ControlFlowOp),

    /// Instructions setting register values
    // 0b00_001_***
    Put(PutOp),

    /// Instructions moving and swapping register values
    // 0b00_010_***
    Move(MoveOp),

    /// Instructions comparing register values
    // 0b00_011_***
    Cmp(CmpOp),

    /// Arithmetic instructions
    // 0b00_100_***
    Arithmetic(ArithmeticOp),

    /// Bit operations & boolean algebra instructions
    // 0b00_101_***
    Bitwise(BitwiseOp),

    /// Operations on byte strings
    // 0b00_110_***
    Bytes(BytesOp),

    /// Cryptographic hashing functions
    // 0b01_000_***
    Digest(DigestOp),

    #[cfg(feature = "secp256k1")]
    /// Operations on Secp256k1 elliptic curve
    // 0b01_001_0**
    Secp256k1(Secp256k1Op),

    #[cfg(feature = "curve25519")]
    /// Operations on Curve25519 elliptic curve
    // 0b01_001_1**
    Curve25519(Curve25519Op),

    /// Extension operations which can be provided by a host environment
    // 0b10_***_***
    ExtensionCodes(Extension),

    // Reserved operations for future use.
    //
    // When such an opcode is met in the bytecode the decoder MUST fail.
    // 0x11_***_***
    /// No-operation instruction
    // #[value = 0b11_111_111]
    Nop,
}

/// Control-flow instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum ControlFlowOp {
    /// Completes program execution writing `false` to `st0` (indicating
    /// program failure)
    #[display("fail")]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program
    /// success)
    #[display("succ")]
    Succ,

    /// Unconditionally jumps to an offset. Increments `cy0`.
    #[display("jmp\t\t{0:#06X}")]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments
    /// `cy0`.
    #[display("jif\t\t{0:#06X}")]
    Jif(u16),

    /// Jumps to other location in the current code with ability to return
    /// back (calls a subroutine). Increments `cy0` and pushes offset of the
    /// instruction which follows current one to `cs0`.
    #[display("routine\t{0:#06X}")]
    Routine(u16),

    /// Calls code from an external library identified by the hash of its code.
    /// Increments `cy0` and `cp0` and pushes offset of the instruction which
    /// follows current one to `cs0`.
    #[display("call\t{0}")]
    Call(LibSite),

    /// Passes execution to other library without an option to return.
    /// Does not increments `cy0` and `cp0` counters and does not add anything
    /// to the call stack `cs0`.
    #[display("exec\t{0}")]
    Exec(LibSite),

    /// Returns execution flow to the previous location from the top of `cs0`.
    /// Does not change value in `cy0`. Decrements `cp0`.
    #[display("ret")]
    Ret,
}

/// Instructions setting register values
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
pub enum PutOp {
    /// Sets `a` register value to zero
    #[display("zero\t{0}{1}")]
    ZeroA(RegA, Reg32),

    /// Sets `r` register value to zero
    #[display("zero\t{0}{1}")]
    ZeroR(RegR, Reg32),

    /// Cleans a value of `a` register (sets it to undefined state)
    #[display("cl\t\t{0}{1}")]
    ClA(RegA, Reg32),

    /// Cleans a value of `r` register (sets it to undefined state)
    #[display("cl\t\t{0}{1}")]
    ClR(RegR, Reg32),

    /// Unconditionally assigns a value to `a` register
    #[display("put\t\t{0}{1}, {2}")]
    PutA(RegA, Reg32, Value),

    /// Unconditionally assigns a value to `r` register
    #[display("put\t\t{0}{1}, {2}")]
    PutR(RegR, Reg32, Value),

    /// Conditionally assigns a value to `a` register if the register is in
    /// uninitialized state
    #[display("putif\t{0}{1}, {2}")]
    PutIfA(RegA, Reg32, Value),

    /// Conditionally assigns a value to `r` register if the register is in
    /// uninitialized state
    #[display("putif\t{0}{1}, {2}")]
    PutIfR(RegR, Reg32, Value),
}

/// Instructions moving and swapping register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum MoveOp {
    /// Swap operation for arithmetic registers. If the value does not fit
    /// destination bit dimensions truncates the most significant bits until
    /// they fit.
    #[display("swp\t\t{0}{1},{2}{3}")]
    SwpA(RegA, Reg32, RegA, Reg32),

    /// Swap operation for non-arithmetic registers. If the value does not fit
    /// destination bit dimensions truncates the most significant bits until
    /// they fit.
    #[display("swp\t\t{0}{1},{2}{3}")]
    SwpR(RegR, Reg32, RegR, Reg32),

    /// Swap operation between arithmetic and non-arithmetic registers. If the
    /// value does not fit destination bit dimensions truncates the most
    /// significant bits until they fit.
    #[display("swp\t\t{0}{1},{2}{3}")]
    SwpAR(RegA, Reg32, RegR, Reg32),

    /// Array move operation: duplicates values of all register set into
    /// another set
    #[display("amov:{2}\t{0},{1}")]
    AMov(RegA, RegA, NumType),

    /// Move operation: duplicates value of one of the arithmetic registers
    /// into another arithmetic register
    #[display("mov\t\t{0}{1},{2}{3}")]
    MovA(RegA, Reg32, RegA, Reg32),

    /// Move operation: duplicates value of one of the non-arithmetic registers
    /// into another non-arithmetic register
    #[display("mov\t\t{0}{1},{2}{3}")]
    MovR(RegR, Reg32, RegR, Reg32),

    /// Move operation: duplicates value of one of the arithmetic registers
    /// into non-arithmetic register
    #[display("mov\t\t{0}{1},{2}{3}")]
    MovAR(RegA, Reg32, RegR, Reg32),

    /// Move operation: duplicates value of one of the n on-arithmetic
    /// registers into arithmetic register
    #[display("mov\t\t{0}{1},{2}{3}")]
    MovRA(RegR, Reg32, RegA, Reg32),
}

/// Instructions comparing register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum CmpOp {
    /// Compares value of two registers setting `st0` to `true` if the first
    /// parameter is greater (and not equal) than the second one. Ignores first
    /// argument if `R` register is used.
    #[display("gt:{0}\t\t{1}{2},{1}{3}")]
    GtA(NumType, RegA, Reg32, Reg32),

    /// Compares value of two registers setting `st0` to `true` if the first
    /// parameter is greater (and not equal) than the second one. Treats both
    /// values as unsigned integers
    #[display("gt\t\t{0}{1},{2}{3}")]
    GtR(RegR, Reg16, RegR, Reg32),

    /// Compares value of two registers setting `st0` to `true` if the first
    /// parameter is smaller (and not equal) than the second one. Ignores first
    /// argument if `R` register is used.
    #[display("lt:{0}\t\t{1}{2},{1}{3}")]
    LtA(NumType, RegA, Reg32, Reg32),

    /// Compares value of two registers setting `st0` to `true` if the first
    /// parameter is smaller (and not equal) than the second one. Treats both
    /// values as unsigned integers
    #[display("lt\t\t{0}{1},{2}{3}")]
    LtR(RegR, Reg16, RegR, Reg32),

    /// Checks equality of value in two arithmetic (`A`) registers putting
    /// result into `st0`
    #[display("eq\t\t{0}{1},{2}{3}")]
    EqA(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting
    /// result into `st0`
    #[display("eq\t\t{0}{1},{2}{3}")]
    EqR(RegR, Reg32, RegR, Reg32),

    /// Measures bit length of a value in one of the registers putting result
    /// to `a16[0]`. If the register is in uninitialized state sets `a16[0]` to
    /// be uninitialized as well.
    #[display("len\t\t{0}{1}")]
    Len(RegA, Reg32),

    /// Counts number of `1` bits in register putting result to `a16[0]`
    /// register. If the register is in uninitialized state sets `a16[0]` to be
    /// uninitialized as well.
    #[display("cnt\t\t{0}{1}")]
    Cnt(RegA, Reg32),

    /// Assigns value of `a8[0]` register to `st0`.
    #[display("st2a")]
    St2A,

    /// `st0` value of `st0` register to the result of `a8[0] != 0`. If the
    /// value is not set, assigns `st0` `false`
    #[display("a2st")]
    A2St,
}

/// Arithmetic instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum ArithmeticOp {
    /// Negates most significant bit
    #[display("neg\t\t{0}{1}")]
    Neg(RegA, Reg32),

    /// Increases register value on a given step.
    #[display("{0}:{1}\t{2}{3},{4}")]
    Stp(IncDec, Arithmetics, RegA, Reg32, u4),

    /// Adds two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[display("add:{0}\t{1}{2},{1}{3}")]
    Add(Arithmetics, RegA, Reg32, Reg32),

    /// Subtracts two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[display("sub:{0}\t{1}{2},{1}{3}")]
    Sub(Arithmetics, RegA, Reg32, Reg32),

    /// Multiplies two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[display("mul:{0}\t{1}{2},{1}{3}")]
    Mul(Arithmetics, RegA, Reg32, Reg32),

    /// Divides two registers. Puts result to `a_[0]` or `ap[0]`, if
    /// [`Arithmetics::IntArbitraryPrecision`] or
    /// [`Arithmetics::FloatArbitraryPrecision`] is used
    #[display("div:{0}\t{1}{2},{1}{3}")]
    Div(Arithmetics, RegA, Reg32, Reg32),

    /// Modulo division
    #[display("rem:{0}\t{1}{2},{1}{3}")]
    Rem(Arithmetics, RegA, Reg32, Reg32),

    /// Puts absolute value of register into `a8[0]`
    #[display("abs\t\t{0}{1}")]
    Abs(RegA, Reg32),
}

/// Bit operations & boolean algebra instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum BitwiseOp {
    /// Bitwise AND operation
    #[display("and\t\t{0}{1},{0}{2},{0}{3}")]
    And(
        RegA,
        Reg32,
        Reg32,
        /// Operation destination, only first 8 registers
        Reg8,
    ),

    /// Bitwise OR operation
    #[display("or\t\t{0}{1},{0}{2},{0}{3}")]
    Or(RegA, Reg32, Reg32, Reg8),

    /// Bitwise XOR operation
    #[display("xor\t\t{0}{1},{0}{2},{0}{3}")]
    Xor(RegA, Reg32, Reg32, Reg8),

    /// Bitwise inversion
    #[display("not\t\t{0}{1}")]
    Not(RegA, Reg32),

    /// Left bit shift, filling added bits values with zeros
    #[display("shl\t\t{0}{1},a8{2},{0}{3}")]
    Shl(RegA, Reg32, Reg32 /* Always `a8` */, Reg8),

    /// Right bit shift, filling added bits values with zeros
    #[display("shr\t\t{0}{1},a8{2},{0}{3}")]
    Shr(RegA, Reg32, Reg32, Reg8),

    /// Left bit shift, cycling the shifted values (most significant bit
    /// becomes least significant)
    #[display("scl\t\t{0}{1},a8{2},{0}{3}")]
    Scl(RegA, Reg32, Reg32, Reg8),

    /// Right bit shift, cycling the shifted values (least significant bit
    /// becomes nost significant)
    #[display("scr\t\t{0}{1},a8{2},{0}{3}")]
    Scr(RegA, Reg32, Reg32, Reg8),
}

/// Operations on byte strings
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum BytesOp {
    /// Put bytestring into a byte string register
    #[display("put\t\ts16[{0}],{1}")]
    Put(/** `s` register index */ u8, Blob),

    /// Move bytestring value between registers
    #[display("mov\t\ts16[{0}],s16[{1}]")]
    Mov(/** `s` register index */ u8, /** `s` register index */ u8),

    /// Swap bytestring value between registers
    #[display("swp\t\ts16[{0}],s16[{1}]")]
    Swp(/** `s` register index */ u8, /** `s` register index */ u8),

    /// Fill segment of bytestring with specific byte value
    #[display("fill\ts16[{0}],{1}..{2},{3}")]
    Fill(
        /** `s` register index */ u8,
        /** from */ u16,
        /** to */ u16,
        /** value */ u8,
    ),

    /// Put length of the string into `a16[0]` register
    #[display("len\t\ts16[{0}],a16[0]")]
    LenS(/** `s` register index */ u8),

    /// Count number of byte occurrences within the string and stores
    /// that value in `a16[0]`
    #[display("count\ts16[{0}],{1},a16[0]")]
    Count(/** `s` register index */ u8, /** byte to count */ u8),

    /// Compare two strings from two registers, putting result into `cm0`
    #[display("cmp\t\ts16[{0}],s16[{0}]")]
    Cmp(u8, u8),

    /// Compute length of the fragment shared between two strings
    #[display("comm\ts16[{0}],s16[{1}]")]
    Comm(u8, u8),

    /// Count number of occurrences of one string within another putting
    /// result to `a16[0]`
    #[display("find\ts16[{0}],s16[{1}],a16[0]")]
    Find(
        /** `s` register with string */ u8,
        /** `s` register with matching fragment */ u8,
    ),

    /// Extract byte string slice into `a` or `r` register
    #[display("extr\ts16{0},a16{1},{2}{3}")]
    Extr(
        /** `s` register index */ Reg32,
        /** `a16` register with offset */ Reg32,
        RegBlock,
        Reg32,
    ),

    /// Inject a `a` or `r` value at a given position to string register,
    /// replacing value of the corresponding bytes.
    #[display("extr\ts16{0},a16{1},{2}{3}")]
    Inj(
        /** `s` register index */ Reg32,
        /** `a16` register with offset */ Reg32,
        RegBlock,
        Reg32,
    ),

    /// Join bytestrings from two registers
    #[display("join\ts16[{0}],s16[{1}],s16[{2}]")]
    Join(
        /** Source 1 */ u8,
        /** Source 2 */ u8,
        /** Destination */ u8,
    ),

    /// Split bytestring at a given index into two registers
    #[display("split\ts16[{0}],{1},s16[{2}],s16[{3}]")]
    Split(
        /** Source */ u8,
        /** Offset */ u16,
        /** Destination 1 */ u8,
        /** Destination 2 */ u8,
    ),

    /// Insert value from one of bytestring register at a given index of other
    /// bytestring register, shifting string bytes. If destination register
    /// does not fits the length of the new string, its final bytes are
    /// removed.
    #[display("ins\t\ts16[{0}],s16[{1}],{2}")]
    Ins(
        /** Insert from register */ u8,
        /** Insert to register */ u8,
        /** Offset for insert place */ u16,
    ),

    /// Delete bytes in a given range, shifting the remaining bytes
    #[display("ins\t\ts16[{0}],{1}..{2}")]
    Del(
        /** Register index */ u8,
        /** Delete from */ u16,
        /** Delete to */ u16,
    ),

    /// Extract fragment of bytestring into a register
    #[display("transl\ts16[{0}],{1}..{2},s16[{3}]")]
    Transl(
        /** Source */ u8,
        /** Start from */ u16,
        /** End at */ u16,
        /** Index to put translocated portion */ u8,
    ),
}

/// Cryptographic hashing functions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[non_exhaustive]
pub enum DigestOp {
    /// Computes RIPEMD160 hash value
    #[display("ripemd\ts16{0},r160{1}")]
    Ripemd(
        /** Index of string register */ Reg32,
        /** Index of `r160` register to save result to */ Reg8,
    ),

    /// Computes SHA256 hash value
    #[display("sha256\ts16{0},r256{1}")]
    Sha256(
        /** Index of string register */ Reg32,
        /** Index of `r256` register to save result to */ Reg8,
    ),

    /// Computes SHA256 hash value
    #[display("sha512\ts16{0},r512{1}")]
    Sha512(
        /** Index of string register */ Reg32,
        /** Index of `r512` register to save result to */ Reg8,
    ),
}

/// Operations on Secp256k1 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Secp256k1Op {
    /// Generates new elliptic curve point value saved into destination
    /// register in `r512` set using scalar value from the source `r256`
    /// register
    #[display("secpgen\tr256{0},r512{1}")]
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),

    /// Multiplies elliptic curve point on a scalar
    #[display("secpmul\t{0}256{1},r512{2},r512{3}")]
    Mul(
        /** Use `a` or `r` register as scalar source */ RegBlock,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),

    /// Adds two elliptic curve points
    #[display("secpadd\tr512{0},r512{1}")]
    Add(/** Source 1 */ Reg32, /** Source 2 and destination */ Reg8),

    /// Negates elliptic curve point
    #[display("secpneg\tr512{0},r512{1}")]
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}

/// Operations on Curve25519 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Curve25519Op {
    /// Generates new elliptic curve point value saved into destination
    /// register in `r512` set using scalar value from the source `r256`
    /// register
    #[display("edgen\tr256{0},r512{1}")]
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),

    /// Multiplies elliptic curve point on a scalar
    #[display("edmul\t{0}256{1},r512{2},r512{3}")]
    Mul(
        /** Use `a` or `r` register as scalar source */ RegBlock,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),

    /// Adds two elliptic curve points
    #[display("edadd\tr512{0},r512{1},r512{2},{3}")]
    Add(
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
        /** Allow overflows */ bool,
    ),

    /// Negates elliptic curve point
    #[display("edneg\tr512{0},r512{1}")]
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}
