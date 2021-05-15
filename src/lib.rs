

#[non_exhaustive]
enum Instruction {
    ControlFlow(ControlFlowOp),
    Register(RegisterOp),
    Arithmetic(ArithmeticOp),
    Bitwise(BitwiseOp),
    Digest(DigestOp),
    Secp256k1(SecpOp),
    Ed25519(Ed25519Op),
    // Other curves may be added later
}

enum ControlFlowOp {
    Succ,
    Fail,
    Cmp,
    Jmp,
    Jif,
}

enum RegisterOp {
    Split,
    Join,
    Mov,
}

enum ArithmeticOp {
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Abs,
}

enum BitwiseOp {
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
}

#[non_exhaustive]
enum DigestOp {
    Ripemd,
    Sha2,
    Sha3
}

enum SecpOp {
    Gen,
    Add,
    Neg,
}

enum Ed25519Op {
    Gen,
    Add,
    Neg,
}

#[derive(Debug)]
struct Registers {
    // Arithmetic registers:
    a8: [Option<u8>; 16],
    a16: [Option<u16>; 16],
    a24: [Option<[u8; 3]>; 16],
    a32: [Option<u32>; 16],
    a64: [Option<u64>; 16],
    a128: [Option<u128>; 16],
    a256: [Option<u256>; 16],
    a512: [Option<u512>; 16],

    // Arbitrary-precision
    aap: [Option<Box<[u8]>>; 16],

    // Non-arithmetic registers:
    r160: [Option<[u8; 20]>; 4],
    r256: [Option<[u8; 32]>; 4],
    r512: [Option<[u8; 64]>; 4],
    r4096: [Option<[u8; 512]>; 4],

    // Control flow registers
    cf0: u16,
}

impl Registers {
    pub fn execute(&mut self, code: &[u8]) {

    }
}