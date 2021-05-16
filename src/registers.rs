// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::{u256, u512};

use crate::{Blob, LibSite};

/// All possible register indexes for `a` and `r` register sets
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum Reg32 {
    /// Register with index `[0]`
    #[cfg_attr(feature = "std", display("[0]"))]
    Reg1 = 0,

    /// Register with index `[1]`
    #[cfg_attr(feature = "std", display("[1]"))]
    Reg2 = 1,

    /// Register with index `[2]`
    #[cfg_attr(feature = "std", display("[2]"))]
    Reg3 = 2,

    /// Register with index `[3]`
    #[cfg_attr(feature = "std", display("[3]"))]
    Reg4 = 3,

    /// Register with index `[4]`
    #[cfg_attr(feature = "std", display("[4]"))]
    Reg5 = 4,

    /// Register with index `[5]`
    #[cfg_attr(feature = "std", display("[5]"))]
    Reg6 = 5,

    /// Register with index `[6]`
    #[cfg_attr(feature = "std", display("[6]"))]
    Reg7 = 6,

    /// Register with index `[7]`
    #[cfg_attr(feature = "std", display("[7]"))]
    Reg8 = 7,

    /// Register with index `[8]`
    #[cfg_attr(feature = "std", display("[8]"))]
    Reg9 = 8,

    /// Register with index `[9]`
    #[cfg_attr(feature = "std", display("[9]"))]
    Reg10 = 9,

    /// Register with index `[10]`
    #[cfg_attr(feature = "std", display("[10]"))]
    Reg11 = 10,

    /// Register with index `[11]`
    #[cfg_attr(feature = "std", display("[11]"))]
    Reg12 = 11,

    /// Register with index `[12]`
    #[cfg_attr(feature = "std", display("[12]"))]
    Reg13 = 12,

    /// Register with index `[13]`
    #[cfg_attr(feature = "std", display("[13]"))]
    Reg14 = 13,

    /// Register with index `[14]`
    #[cfg_attr(feature = "std", display("[14]"))]
    Reg15 = 14,

    /// Register with index `[15]`
    #[cfg_attr(feature = "std", display("[15]"))]
    Reg16 = 15,

    /// Register with index `[16]`
    #[cfg_attr(feature = "std", display("[16]"))]
    Reg17 = 16,

    /// Register with index `[17]`
    #[cfg_attr(feature = "std", display("[17]"))]
    Reg18 = 17,

    /// Register with index `[18]`
    #[cfg_attr(feature = "std", display("[18]"))]
    Reg19 = 18,

    /// Register with index `[19]`
    #[cfg_attr(feature = "std", display("[19]"))]
    Reg20 = 19,

    /// Register with index `[20]`
    #[cfg_attr(feature = "std", display("[20]"))]
    Reg21 = 20,

    /// Register with index `[21]`
    #[cfg_attr(feature = "std", display("[21]"))]
    Reg22 = 21,

    /// Register with index `[22]`
    #[cfg_attr(feature = "std", display("[22]"))]
    Reg23 = 22,

    /// Register with index `[23]`
    #[cfg_attr(feature = "std", display("[23]"))]
    Reg24 = 23,

    /// Register with index `[24]`
    #[cfg_attr(feature = "std", display("[24]"))]
    Reg25 = 24,

    /// Register with index `[25]`
    #[cfg_attr(feature = "std", display("[25]"))]
    Reg26 = 25,

    /// Register with index `[26]`
    #[cfg_attr(feature = "std", display("[26]"))]
    Reg27 = 26,

    /// Register with index `[27]`
    #[cfg_attr(feature = "std", display("[27]"))]
    Reg28 = 27,

    /// Register with index `[28]`
    #[cfg_attr(feature = "std", display("[28]"))]
    Reg29 = 28,

    /// Register with index `[29]`
    #[cfg_attr(feature = "std", display("[29]"))]
    Reg30 = 29,

    /// Register with index `[30]`
    #[cfg_attr(feature = "std", display("[30]"))]
    Reg31 = 30,

    /// Register with index `[31]`
    #[cfg_attr(feature = "std", display("[31]"))]
    Reg32 = 31,
}

impl Default for Reg32 {
    fn default() -> Self {
        Reg32::Reg1
    }
}

/// Short version of register indexes for `a` and `r` register sets covering
/// initial 8 registers only
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum Reg8 {
    /// Register with index `[0]`
    #[cfg_attr(feature = "std", display("[0]"))]
    Reg1 = 0,

    /// Register with index `[1]`
    #[cfg_attr(feature = "std", display("[1]"))]
    Reg2 = 1,

    /// Register with index `[2]`
    #[cfg_attr(feature = "std", display("[2]"))]
    Reg3 = 2,

    /// Register with index `[3]`
    #[cfg_attr(feature = "std", display("[3]"))]
    Reg4 = 3,

    /// Register with index `[4]`
    #[cfg_attr(feature = "std", display("[4]"))]
    Reg5 = 4,

    /// Register with index `[5]`
    #[cfg_attr(feature = "std", display("[5]"))]
    Reg6 = 5,

    /// Register with index `[6]`
    #[cfg_attr(feature = "std", display("[6]"))]
    Reg7 = 6,

    /// Register with index `[7]`
    #[cfg_attr(feature = "std", display("[7]"))]
    Reg8 = 7,
}

impl Default for Reg8 {
    fn default() -> Self {
        Reg8::Reg1
    }
}

/// Enumeration of the `a` set of registers (arithmetic registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum RegA {
    /// Arbitrary-precision register
    #[cfg_attr(feature = "std", display("ap"))]
    AP,

    /// 8-bit arithmetics register
    #[cfg_attr(feature = "std", display("a8"))]
    A8,

    /// 16-bit arithmetics register
    #[cfg_attr(feature = "std", display("a16"))]
    A16,

    /// 32-bit arithmetics register
    #[cfg_attr(feature = "std", display("a32"))]
    A32,

    /// 64-bit arithmetics register
    #[cfg_attr(feature = "std", display("a64"))]
    A64,

    /// 128-bit arithmetics register
    #[cfg_attr(feature = "std", display("a128"))]
    A128,

    /// 256-bit arithmetics register
    #[cfg_attr(feature = "std", display("a256"))]
    A256,

    /// 512-bit arithmetics register
    #[cfg_attr(feature = "std", display("a512"))]
    A512,
}

/// Enumeration of the `r` set of registers (non-arithmetic registers, mostly
/// used for cryptography)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display), display(Debug))]
pub enum RegR {
    /// 128-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r128"))]
    R128,

    /// 160-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r160"))]
    R160,

    /// 256-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r256"))]
    R256,

    /// 512-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r512"))]
    R512,

    /// 1024-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r1024"))]
    R1024,

    /// 2048-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r2048"))]
    R2048,

    /// 4096-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r4096"))]
    R4096,

    /// 8192-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r8192"))]
    R8192,
}

/// All non-string registers directly accessible by AluVM instructions,
/// consisting of `a` and `r` sets of registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display), display(inner))]
pub enum Reg {
    /// Arithmetic registers (`a` registers)
    A(RegA),

    /// Non-arithmetic (generic) registers (`a` registers)
    R(RegR),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Registers {
    /// Arbitrary-precision arithmetics registers
    pub(crate) ap: [Option<[u8; 1024]>; 32],

    // Arithmetic registers:
    pub(crate) a8: [Option<u8>; 32],
    pub(crate) a16: [Option<u16>; 32],
    pub(crate) a32: [Option<u32>; 32],
    pub(crate) a64: [Option<u64>; 32],
    pub(crate) a128: [Option<u128>; 32],
    pub(crate) a256: [Option<u256>; 32],
    pub(crate) a512: [Option<u512>; 32],

    // Non-arithmetic registers:
    pub(crate) r128: [Option<[u8; 16]>; 32],
    pub(crate) r160: [Option<[u8; 20]>; 32],
    pub(crate) r256: [Option<[u8; 32]>; 32],
    pub(crate) r512: [Option<[u8; 64]>; 32],
    pub(crate) r1024: [Option<[u8; 128]>; 32],
    pub(crate) r2048: [Option<[u8; 256]>; 32],
    pub(crate) r4096: [Option<[u8; 512]>; 32],
    pub(crate) r8192: [Option<[u8; 1024]>; 32],

    /// String and bytestring registers
    pub(crate) s16: [Option<(u16, [u8; u16::MAX as usize])>; u8::MAX as usize],

    /// Control flow register which stores result of equality and other types
    /// of boolean checks. Initialized with `true`
    pub(crate) st0: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is
    /// limited by 2^16 per script.
    cy0: u16,

    /// Call stack. Maximal size is `u16::MAX` (limited by `cy0` mechanics and
    /// `cp0`)
    cs0: [LibSite; u16::MAX as usize],

    /// Defines "top" of the call stack
    cp0: u16,
}

impl Default for Registers {
    #[inline]
    fn default() -> Self {
        Registers {
            ap: Default::default(),
            a8: Default::default(),
            a16: Default::default(),
            a32: Default::default(),
            a64: Default::default(),
            a128: Default::default(),
            a256: Default::default(),
            a512: Default::default(),

            r128: Default::default(),
            r160: Default::default(),
            r256: Default::default(),
            r512: Default::default(),
            r1024: Default::default(),
            r2048: Default::default(),
            r4096: Default::default(),
            r8192: Default::default(),

            st0: true,
            cy0: 0,
            cs0: [LibSite::default(); u16::MAX as usize],
            s16: [None; u8::MAX as usize],
            cp0: 0,
        }
    }
}

impl Registers {
    #[inline]
    pub fn new() -> Registers {
        Registers::default()
    }

    pub(crate) fn jmp(&mut self) -> Result<(), ()> {
        self.cy0
            .checked_add(1)
            .ok_or_else(|| {
                self.st0 = false;
            })
            .map(|_| ())
    }

    pub(crate) fn call(&mut self, site: LibSite) -> Result<(), ()> {
        self.cy0
            .checked_add(1)
            .ok_or_else(|| {
                self.st0 = false;
            })
            .and_then(|_| {
                self.cp0.checked_add(1).ok_or_else(|| {
                    self.st0 = false;
                })
            })
            .map(|_| {
                self.cs0[self.cp0 as usize - 1] = site;
            })
    }

    pub(crate) fn ret(&mut self) -> Option<LibSite> {
        if self.cp0 == 0 {
            None
        } else {
            self.cs0[self.cp0 as usize] = LibSite::default();
            self.cp0 -= 1;
            Some(self.cs0[self.cp0 as usize])
        }
    }

    pub fn get(&self, reg: Reg, index: Reg32) -> Option<Blob> {
        let index = index as usize;
        match reg {
            Reg::A(a) => match a {
                RegA::AP => self.ap[index].map(Blob::from),
                RegA::A8 => self.a8[index].map(Blob::from),
                RegA::A16 => self.a16[index].map(Blob::from),
                RegA::A32 => self.a32[index].map(Blob::from),
                RegA::A64 => self.a64[index].map(Blob::from),
                RegA::A128 => self.a128[index].map(Blob::from),
                RegA::A256 => self.a256[index].map(Blob::from),
                RegA::A512 => self.a512[index].map(Blob::from),
            },

            Reg::R(r) => match r {
                RegR::R128 => self.r128[index].map(Blob::from),
                RegR::R160 => self.r160[index].map(Blob::from),
                RegR::R256 => self.r256[index].map(Blob::from),
                RegR::R512 => self.r512[index].map(Blob::from),
                RegR::R1024 => self.r1024[index].map(Blob::from),
                RegR::R2048 => self.r2048[index].map(Blob::from),
                RegR::R4096 => self.r4096[index].map(Blob::from),
                RegR::R8192 => self.r8192[index].map(Blob::from),
            },
        }
    }

    pub fn set(&mut self, reg: Reg, index: Reg32, value: Option<Blob>) {
        let index = index as usize;
        match reg {
            Reg::A(a) => match a {
                RegA::AP => self.ap[index] = value.map(Blob::into),
                RegA::A8 => self.a8[index] = value.map(Blob::into),
                RegA::A16 => self.a16[index] = value.map(Blob::into),
                RegA::A32 => self.a32[index] = value.map(Blob::into),
                RegA::A64 => self.a64[index] = value.map(Blob::into),
                RegA::A128 => self.a128[index] = value.map(Blob::into),
                RegA::A256 => self.a256[index] = value.map(Blob::into),
                RegA::A512 => self.a512[index] = value.map(Blob::into),
            },
            Reg::R(r) => match r {
                RegR::R128 => self.r128[index] = value.map(Blob::into),
                RegR::R160 => self.r160[index] = value.map(Blob::into),
                RegR::R256 => self.r256[index] = value.map(Blob::into),
                RegR::R512 => self.r512[index] = value.map(Blob::into),
                RegR::R1024 => self.r1024[index] = value.map(Blob::into),
                RegR::R2048 => self.r2048[index] = value.map(Blob::into),
                RegR::R4096 => self.r4096[index] = value.map(Blob::into),
                RegR::R8192 => self.r8192[index] = value.map(Blob::into),
            },
        }
    }

    pub fn status(&self) -> bool {
        return self.st0;
    }
}
