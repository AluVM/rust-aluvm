use std::cmp::Ordering;

#[non_exhaustive]
pub enum Instruction {
    #[value = 0b00_000_000]
    ControlFlow(ControlFlowOp),

    #[value = 0b00_001_000]
    Register(RegisterOp),

    #[value = 0b00_010_000]
    Arithmetic(ArithmeticOp),

    #[value = 0b00_011_000]
    Bitwise(BitwiseOp),

    #[value = 0b00_100_000]
    Digest(DigestOp),

    #[value = 0b00_101_000]
    Secp256k1(SecpOp),

    #[value = 0b00_101_100]
    Ed25519(Ed25519Op),
}

pub enum ControlFlowOp {
    /// Completes program execution writing `false` to `st0` (indicating program failure)
    #[value = 0b000]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program success)
    #[value = 0b001]
    Succ,

    /// Compares value of two arithmetic (`A`) registers putting result into `cm0`
    #[value = 0b110] // 3 + 4 + 4 => 11 bits
    Cmpa(RegA, Reg16, Reg16),

    /// Compares value of two non-arithmetic (`R`) registers putting result into `cm0`
    #[value = 0b111] // 2 * 4 * 4 = 5 bits
    Cmpr(RegR, Reg4, Reg4),

    /// Checks equality of value in two arithmetic (`A`) registers putting result into `st0`
    #[value = 0b100]
    Eqa(RegA, Reg16, Reg16),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting result into `st0`
    #[value = 0b101]
    Eqr(RegR, Reg4, Reg4),

    /// Unconditionally jumps to an offset. Increments `cy0`.
    #[value = 0b010]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments `cy0`.
    #[value = 0b011]
    Jif(u16),
}

pub enum RegisterOp {
    Split,
    Join,
    /// Puts a value into a register
    Put(RegA, Reg16),
    Mova,
    Movap,
    Movr,

    /// Cleans register value
    Cln,
}

pub enum ArithmeticOp {
    Neg(RegA, Reg16, bool), // 3 + 4 + 1 = 8 bits
    Add(Arithmetics, RegA, Reg16, Reg16), // 3 + 3 + 4 + 4  => 14 bits
    Sub(Arithmetics, RegA, Reg16, Reg16),
    Mul(Arithmetics, RegA, Reg16, Reg16),
    Div(Arithmetics, RegA, Reg16, Reg16),
    Mod(RegA, Reg16, bool /** Put the result into `A1` */), // 3 + 4 + 1 = 8 bits
    Abs(RegA, Reg16, RegA, Reg16), // 3 + 4 + 3 + 4 => 14 bits
}

pub enum BitwiseOp {
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,

    /// Shift-cycle left
    Scl,

    /// Shift-cycle right
    Scr,
}

#[non_exhaustive]
pub enum DigestOp {
    Ripemd,
    Sha2,
    Sha3
}

pub enum SecpOp {
    Gen,
    Add,
    Neg,
}

pub enum Ed25519Op {
    Gen,
    Add,
    Neg,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum Reg4 {
    Reg1,
    Reg2,
    Reg3,
    Reg4,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum Reg16 {
    Reg1,
    Reg2,
    Reg3,
    Reg4,
    Reg5,
    Reg6,
    Reg7,
    Reg8,
    Reg9,
    Reg10,
    Reg11,
    Reg12,
    Reg13,
    Reg14,
    Reg15,
    Reg16,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum RegA {
    AP,
    A8,
    A16,
    A32,
    A64,
    A128,
    A256,
    A512,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum RegR {
    R160,
    R256,
    R512,
    R1024,
}

pub enum Arithmetics {
    IntChecked(bool),
    IntUnchecked(bool),
    IntArbitraryPrecision(bool),
    Float,
    FloatArbitraryPrecision,
}

#[derive(Debug)]
struct Registers {
    // Arithmetic registers:
    a8: [Option<u8>; 16],
    a16: [Option<u16>; 16],
    a32: [Option<u32>; 16],
    a64: [Option<u64>; 16],
    a128: [Option<u128>; 16],
    a256: [Option<u256>; 16],
    a512: [Option<u512>; 16],

    /// Arbitrary-precision arithmetics registers
    ap: [Option<Box<[u8]>>; 16],

    // Non-arithmetic registers:
    r160: [Option<[u8; 20]>; 4],
    r256: [Option<[u8; 32]>; 4],
    r512: [Option<[u8; 64]>; 4],
    r4096: [Option<[u8; 512]>; 4],

    /// Control flow register which stores result of comparison operations. Initialized with `0`
    cm0: Ordering,

    /// Control flow register which stores result of equality and other types of boolean checks. Initialized with `true`
    st0: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is limited by 2^16 per script.
    cy0: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            st0: true,
            cm0: Ordering::Equal,
            ..Default::default()
        }
    }
}

impl Registers {
    pub fn execute(&mut self, code: &[u8]) {

    }
}