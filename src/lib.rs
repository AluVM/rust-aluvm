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
    #[value = 0b110] // 3 + 5 + 3 + 5 => 16 bits
    Cmpa(RegA, Reg32, RegA, Reg32),

    /// Compares value of two non-arithmetic (`R`) registers putting result into `cm0`
    #[value = 0b111]
    Cmpr(RegR, Reg32, RegR, Reg32),

    /// Checks equality of value in two arithmetic (`A`) registers putting result into `st0`
    #[value = 0b100]
    Eqa(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting result into `st0`
    #[value = 0b101]
    Eqr(RegR, Reg32, RegR, Reg32),

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
    Put(RegA, Reg32),

    Mova,
    Movr,

    Swpa,
    Swpr,

    /// Cleans register value
    Cln,
}

pub enum ArithmeticOp {
    Neg(RegA, Reg32), // 3 + 5 = 8 bits
    Add(Arithmetics, RegA, Reg32, Reg32), // 3 + 3 + 5 + 5  => 16 bits
    Sub(Arithmetics, RegA, Reg32, Reg32),
    Mul(Arithmetics, RegA, Reg32, Reg32),
    Div(Arithmetics, RegA, Reg32, Reg32),
    Mod(RegA, Reg32), // 3 + 5 = 8 bits
    Abs(RegA, Reg32, RegA, Reg32), // 3 + 5 + 3 + 5 => 16 bits
}

pub enum BitwiseOp {
    And(RegA, Reg32, Reg32, Reg8 /** Operation destination, only first 8 registers */),
    Or(RegA, Reg32, Reg32, Reg8),
    Xor(RegA, Reg32, Reg32, Reg8),

    Not(RegA, Reg32),

    Shl(RegA, Reg32, Reg32 /** Always `a8` */, Reg8),
    Shr(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle left
    Scl(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle right
    Scr(RegA, Reg32, Reg32, Reg8),
}

#[non_exhaustive]
pub enum DigestOp {
    Ripemd(
        Reg32 /** Which of `a16` registers contain start offset */,
        Reg32 /** Index of string register */,
        Reg32 /** Index of `r160` register to save result to */,
        bool /** Clear string register after operation */
    ),
    Sha2(
        Reg32 /** Which of `a16` registers contain start offset */,
        Reg32 /** Index of string register */,
        Reg32 /** Index of `r160` register to save result to */,
        bool /** Clear string register after operation */
    ),
}

pub enum SecpOp {
    Gen(Reg8),
    Mul(
        bool, /** Use `a` or `r` register as scalar source */
        Reg32, /** Scalar register index */
        Reg32, /** Source `r` register index containing EC point */
        Reg32, /** Destination `r` register index */
    ),
    Add(
        bool, /** Allow overflows */
        Reg32, /** Source 1 */
        Reg32, /** Source 2 */
        Reg32, /** Source 3 */
    ),
    Neg(
        Reg32,
        Reg8,
    ),
}

pub enum Ed25519Op {
    Gen,
    Mul,
    Add,
    Neg,
}

#[derive(Debug, Display)]
#[display(Debug)]
pub enum Reg32 {
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
    Reg17,
    Reg18,
    Reg19,
    Reg20,
    Reg21,
    Reg22,
    Reg23,
    Reg24,
    Reg25,
    Reg26,
    Reg27,
    Reg28,
    Reg29,
    Reg30,
    Reg31,
    Reg32,
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
    R128,
    R160,
    R256,
    R512,
    R1024,
    R2048,
    R4096,
    R8192,
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
    a8: [Option<u8>; 32],
    a16: [Option<u16>; 32],
    a32: [Option<u32>; 32],
    a64: [Option<u64>; 32],
    a128: [Option<u128>; 32],
    a256: [Option<u256>; 32],
    a512: [Option<u512>; 32],

    /// Arbitrary-precision arithmetics registers
    ap: [Option<Box<[u8]>>; 32],

    // Non-arithmetic registers:
    r128: [Option<[u8; 16]>; 32],
    r160: [Option<[u8; 20]>; 32],
    r256: [Option<[u8; 32]>; 32],
    r512: [Option<[u8; 64]>; 32],
    r1024: [Option<[u8; 128]>; 32],
    r2048: [Option<[u8; 256]>; 32],
    r4096: [Option<[u8; 512]>; 32],
    r8192: [Option<[u8; 1024]>; 32],

    /// String and bytestring registers
    s16: [Option<[u8; u16::MAX as usize]>; 32],

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