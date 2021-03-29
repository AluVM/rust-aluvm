

#[derive(Debug)]
struct Registers {
    // Arithmetic registers:
    a8: [Option<u8>; 16],
    a16: [Option<u16>; 16],
    a24: [Option<[u8; 3]>; 16],
    a32: [Option<u32>; 16],
    a64: [Option<u64>; 16],
    a128: [Option<u128>; 16],
    a256: [Option<Uint256>; 16],

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
