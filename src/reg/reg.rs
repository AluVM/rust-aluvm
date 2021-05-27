// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify_num::{u256, u3, u4, u5, u512};
use core::ops::Deref;
use std::collections::BTreeMap;

use crate::{reg::Value, LibSite, RegVal};

/// All possible register indexes for `a` and `r` register sets
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum Reg32 {
    /// Register with index `[1]`
    #[cfg_attr(feature = "std", display("[1]"))]
    Reg1 = 0,

    /// Register with index `[2]`
    #[cfg_attr(feature = "std", display("[2]"))]
    Reg2 = 1,

    /// Register with index `[3]`
    #[cfg_attr(feature = "std", display("[3]"))]
    Reg3 = 2,

    /// Register with index `[4]`
    #[cfg_attr(feature = "std", display("[4]"))]
    Reg4 = 3,

    /// Register with index `[5]`
    #[cfg_attr(feature = "std", display("[5]"))]
    Reg5 = 4,

    /// Register with index `[6]`
    #[cfg_attr(feature = "std", display("[6]"))]
    Reg6 = 5,

    /// Register with index `[7]`
    #[cfg_attr(feature = "std", display("[7]"))]
    Reg7 = 6,

    /// Register with index `[8]`
    #[cfg_attr(feature = "std", display("[8]"))]
    Reg8 = 7,

    /// Register with index `[9]`
    #[cfg_attr(feature = "std", display("[9]"))]
    Reg9 = 8,

    /// Register with index `[10]`
    #[cfg_attr(feature = "std", display("[10]"))]
    Reg10 = 9,

    /// Register with index `[11]`
    #[cfg_attr(feature = "std", display("[11]"))]
    Reg11 = 10,

    /// Register with index `[12]`
    #[cfg_attr(feature = "std", display("[12]"))]
    Reg12 = 11,

    /// Register with index `[13]`
    #[cfg_attr(feature = "std", display("[13]"))]
    Reg13 = 12,

    /// Register with index `[14]`
    #[cfg_attr(feature = "std", display("[14]"))]
    Reg14 = 13,

    /// Register with index `[15]`
    #[cfg_attr(feature = "std", display("[15]"))]
    Reg15 = 14,

    /// Register with index `[16]`
    #[cfg_attr(feature = "std", display("[16]"))]
    Reg16 = 15,

    /// Register with index `[17]`
    #[cfg_attr(feature = "std", display("[17]"))]
    Reg17 = 16,

    /// Register with index `[18]`
    #[cfg_attr(feature = "std", display("[18]"))]
    Reg18 = 17,

    /// Register with index `[19]`
    #[cfg_attr(feature = "std", display("[19]"))]
    Reg19 = 18,

    /// Register with index `[20]`
    #[cfg_attr(feature = "std", display("[10]"))]
    Reg20 = 19,

    /// Register with index `[21]`
    #[cfg_attr(feature = "std", display("[21]"))]
    Reg21 = 20,

    /// Register with index `[22]`
    #[cfg_attr(feature = "std", display("[22]"))]
    Reg22 = 21,

    /// Register with index `[23]`
    #[cfg_attr(feature = "std", display("[23]"))]
    Reg23 = 22,

    /// Register with index `[24]`
    #[cfg_attr(feature = "std", display("[24]"))]
    Reg24 = 23,

    /// Register with index `[25]`
    #[cfg_attr(feature = "std", display("[25]"))]
    Reg25 = 24,

    /// Register with index `[26]`
    #[cfg_attr(feature = "std", display("[26]"))]
    Reg26 = 25,

    /// Register with index `[27]`
    #[cfg_attr(feature = "std", display("[27]"))]
    Reg27 = 26,

    /// Register with index `[28]`
    #[cfg_attr(feature = "std", display("[28]"))]
    Reg28 = 27,

    /// Register with index `[29]`
    #[cfg_attr(feature = "std", display("[29]"))]
    Reg29 = 28,

    /// Register with index `[30]`
    #[cfg_attr(feature = "std", display("[30]"))]
    Reg30 = 29,

    /// Register with index `[31]`
    #[cfg_attr(feature = "std", display("[31]"))]
    Reg31 = 30,

    /// Register with index `[32]`
    #[cfg_attr(feature = "std", display("[32]"))]
    Reg32 = 31,
}

impl Default for Reg32 {
    fn default() -> Self {
        Reg32::Reg1
    }
}

impl From<&Reg32> for u5 {
    fn from(reg32: &Reg32) -> Self {
        u5::with(*reg32 as u8)
    }
}

impl From<Reg32> for u5 {
    fn from(reg32: Reg32) -> Self {
        u5::with(reg32 as u8)
    }
}

impl From<u5> for Reg32 {
    fn from(val: u5) -> Self {
        match val {
            v if v == Reg32::Reg1.into() => Reg32::Reg1,
            v if v == Reg32::Reg2.into() => Reg32::Reg2,
            v if v == Reg32::Reg3.into() => Reg32::Reg3,
            v if v == Reg32::Reg4.into() => Reg32::Reg4,
            v if v == Reg32::Reg5.into() => Reg32::Reg5,
            v if v == Reg32::Reg6.into() => Reg32::Reg6,
            v if v == Reg32::Reg7.into() => Reg32::Reg7,
            v if v == Reg32::Reg8.into() => Reg32::Reg8,
            v if v == Reg32::Reg9.into() => Reg32::Reg9,
            v if v == Reg32::Reg10.into() => Reg32::Reg10,
            v if v == Reg32::Reg11.into() => Reg32::Reg11,
            v if v == Reg32::Reg12.into() => Reg32::Reg12,
            v if v == Reg32::Reg13.into() => Reg32::Reg13,
            v if v == Reg32::Reg14.into() => Reg32::Reg14,
            v if v == Reg32::Reg15.into() => Reg32::Reg15,
            v if v == Reg32::Reg16.into() => Reg32::Reg16,
            v if v == Reg32::Reg17.into() => Reg32::Reg17,
            v if v == Reg32::Reg18.into() => Reg32::Reg18,
            v if v == Reg32::Reg19.into() => Reg32::Reg19,
            v if v == Reg32::Reg20.into() => Reg32::Reg20,
            v if v == Reg32::Reg21.into() => Reg32::Reg21,
            v if v == Reg32::Reg22.into() => Reg32::Reg22,
            v if v == Reg32::Reg23.into() => Reg32::Reg23,
            v if v == Reg32::Reg24.into() => Reg32::Reg24,
            v if v == Reg32::Reg25.into() => Reg32::Reg25,
            v if v == Reg32::Reg26.into() => Reg32::Reg26,
            v if v == Reg32::Reg27.into() => Reg32::Reg27,
            v if v == Reg32::Reg28.into() => Reg32::Reg28,
            v if v == Reg32::Reg29.into() => Reg32::Reg29,
            v if v == Reg32::Reg30.into() => Reg32::Reg30,
            v if v == Reg32::Reg31.into() => Reg32::Reg31,
            v if v == Reg32::Reg32.into() => Reg32::Reg32,
            _ => unreachable!(),
        }
    }
}

/// Shorter version of possible register indexes for `a` and `r` register sets
/// covering initial 16 registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum Reg16 {
    /// Register with index `[1]`
    #[cfg_attr(feature = "std", display("[1]"))]
    Reg1 = 0,

    /// Register with index `[2]`
    #[cfg_attr(feature = "std", display("[2]"))]
    Reg2 = 1,

    /// Register with index `[3]`
    #[cfg_attr(feature = "std", display("[3]"))]
    Reg3 = 2,

    /// Register with index `[4]`
    #[cfg_attr(feature = "std", display("[4]"))]
    Reg4 = 3,

    /// Register with index `[5]`
    #[cfg_attr(feature = "std", display("[5]"))]
    Reg5 = 4,

    /// Register with index `[6]`
    #[cfg_attr(feature = "std", display("[6]"))]
    Reg6 = 5,

    /// Register with index `[7]`
    #[cfg_attr(feature = "std", display("[7]"))]
    Reg7 = 6,

    /// Register with index `[8]`
    #[cfg_attr(feature = "std", display("[8]"))]
    Reg8 = 7,

    /// Register with index `[9]`
    #[cfg_attr(feature = "std", display("[9]"))]
    Reg9 = 8,

    /// Register with index `[10]`
    #[cfg_attr(feature = "std", display("[10]"))]
    Reg10 = 9,

    /// Register with index `[11]`
    #[cfg_attr(feature = "std", display("[11]"))]
    Reg11 = 10,

    /// Register with index `[12]`
    #[cfg_attr(feature = "std", display("[12]"))]
    Reg12 = 11,

    /// Register with index `[13]`
    #[cfg_attr(feature = "std", display("[13]"))]
    Reg13 = 12,

    /// Register with index `[14]`
    #[cfg_attr(feature = "std", display("[14]"))]
    Reg14 = 13,

    /// Register with index `[15]`
    #[cfg_attr(feature = "std", display("[15]"))]
    Reg15 = 14,

    /// Register with index `[16]`
    #[cfg_attr(feature = "std", display("[16]"))]
    Reg16 = 15,
}

impl Default for Reg16 {
    fn default() -> Self {
        Reg16::Reg1
    }
}

impl From<&Reg16> for u4 {
    fn from(reg16: &Reg16) -> Self {
        u4::with(*reg16 as u8)
    }
}

impl From<Reg16> for u4 {
    fn from(reg16: Reg16) -> Self {
        u4::with(reg16 as u8)
    }
}

impl From<u4> for Reg16 {
    fn from(val: u4) -> Self {
        match val {
            v if v == Reg16::Reg1.into() => Reg16::Reg1,
            v if v == Reg16::Reg2.into() => Reg16::Reg2,
            v if v == Reg16::Reg3.into() => Reg16::Reg3,
            v if v == Reg16::Reg4.into() => Reg16::Reg4,
            v if v == Reg16::Reg5.into() => Reg16::Reg5,
            v if v == Reg16::Reg6.into() => Reg16::Reg6,
            v if v == Reg16::Reg7.into() => Reg16::Reg7,
            v if v == Reg16::Reg8.into() => Reg16::Reg8,
            v if v == Reg16::Reg9.into() => Reg16::Reg9,
            v if v == Reg16::Reg10.into() => Reg16::Reg10,
            v if v == Reg16::Reg11.into() => Reg16::Reg11,
            v if v == Reg16::Reg12.into() => Reg16::Reg12,
            v if v == Reg16::Reg13.into() => Reg16::Reg13,
            v if v == Reg16::Reg14.into() => Reg16::Reg14,
            v if v == Reg16::Reg15.into() => Reg16::Reg15,
            v if v == Reg16::Reg16.into() => Reg16::Reg16,
            _ => unreachable!(),
        }
    }
}

impl From<Reg16> for Reg32 {
    fn from(reg16: Reg16) -> Self {
        u5::with(reg16 as u8).into()
    }
}

/// Short version of register indexes for `a` and `r` register sets covering
/// initial 8 registers only
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum Reg8 {
    /// Register with index `[1]`
    #[cfg_attr(feature = "std", display("[1]"))]
    Reg1 = 0,

    /// Register with index `[2]`
    #[cfg_attr(feature = "std", display("[2]"))]
    Reg2 = 1,

    /// Register with index `[3]`
    #[cfg_attr(feature = "std", display("[3]"))]
    Reg3 = 2,

    /// Register with index `[4]`
    #[cfg_attr(feature = "std", display("[4]"))]
    Reg4 = 3,

    /// Register with index `[5]`
    #[cfg_attr(feature = "std", display("[5]"))]
    Reg5 = 4,

    /// Register with index `[6]`
    #[cfg_attr(feature = "std", display("[6]"))]
    Reg6 = 5,

    /// Register with index `[7]`
    #[cfg_attr(feature = "std", display("[7]"))]
    Reg7 = 6,

    /// Register with index `[8]`
    #[cfg_attr(feature = "std", display("[8]"))]
    Reg8 = 7,
}

impl Default for Reg8 {
    fn default() -> Self {
        Reg8::Reg1
    }
}

impl From<&Reg8> for u3 {
    fn from(reg8: &Reg8) -> Self {
        u3::with(*reg8 as u8)
    }
}

impl From<Reg8> for u3 {
    fn from(reg8: Reg8) -> Self {
        u3::with(reg8 as u8)
    }
}

impl From<u3> for Reg8 {
    fn from(val: u3) -> Self {
        match val {
            v if v == Reg8::Reg1.into() => Reg8::Reg1,
            v if v == Reg8::Reg2.into() => Reg8::Reg2,
            v if v == Reg8::Reg3.into() => Reg8::Reg3,
            v if v == Reg8::Reg4.into() => Reg8::Reg4,
            v if v == Reg8::Reg5.into() => Reg8::Reg5,
            v if v == Reg8::Reg6.into() => Reg8::Reg6,
            v if v == Reg8::Reg7.into() => Reg8::Reg7,
            v if v == Reg8::Reg8.into() => Reg8::Reg8,
            _ => unreachable!(),
        }
    }
}

impl From<Reg8> for Reg32 {
    fn from(reg8: Reg8) -> Self {
        u5::with(reg8 as u8).into()
    }
}

/// Enumeration of the `a` set of registers (arithmetic registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
#[repr(u8)]
pub enum RegA {
    /// Arbitrary-precision register
    #[cfg_attr(feature = "std", display("ap"))]
    AP = 0,

    /// 8-bit arithmetics register
    #[cfg_attr(feature = "std", display("a8"))]
    A8 = 1,

    /// 16-bit arithmetics register
    #[cfg_attr(feature = "std", display("a16"))]
    A16 = 2,

    /// 32-bit arithmetics register
    #[cfg_attr(feature = "std", display("a32"))]
    A32 = 3,

    /// 64-bit arithmetics register
    #[cfg_attr(feature = "std", display("a64"))]
    A64 = 4,

    /// 128-bit arithmetics register
    #[cfg_attr(feature = "std", display("a128"))]
    A128 = 5,

    /// 256-bit arithmetics register
    #[cfg_attr(feature = "std", display("a256"))]
    A256 = 6,

    /// 512-bit arithmetics register
    #[cfg_attr(feature = "std", display("a512"))]
    A512 = 7,
}

impl RegA {
    /// Returns bit size, if defined, for the register.
    ///
    /// Bit size is undefined for [`RegA::AP`] register
    pub fn bits(self) -> Option<u16> {
        match self {
            RegA::AP => None,
            RegA::A8 => Some(8),
            RegA::A16 => Some(16),
            RegA::A32 => Some(32),
            RegA::A64 => Some(64),
            RegA::A128 => Some(128),
            RegA::A256 => Some(256),
            RegA::A512 => Some(512),
        }
    }
}

impl From<&RegA> for u3 {
    fn from(rega: &RegA) -> Self {
        u3::with(*rega as u8)
    }
}

impl From<RegA> for u3 {
    fn from(rega: RegA) -> Self {
        u3::with(rega as u8)
    }
}

impl From<u3> for RegA {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegA::AP.into() => RegA::AP,
            v if v == RegA::A8.into() => RegA::A8,
            v if v == RegA::A16.into() => RegA::A16,
            v if v == RegA::A32.into() => RegA::A32,
            v if v == RegA::A64.into() => RegA::A64,
            v if v == RegA::A128.into() => RegA::A128,
            v if v == RegA::A256.into() => RegA::A256,
            v if v == RegA::A512.into() => RegA::A512,
            _ => unreachable!(),
        }
    }
}

/// Enumeration of the `r` set of registers (non-arithmetic registers, mostly
/// used for cryptography)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display), display(Debug))]
#[repr(u8)]
pub enum RegR {
    /// 128-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r128"))]
    R128 = 0,

    /// 160-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r160"))]
    R160 = 1,

    /// 256-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r256"))]
    R256 = 2,

    /// 512-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r512"))]
    R512 = 3,

    /// 1024-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r1024"))]
    R1024 = 4,

    /// 2048-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r2048"))]
    R2048 = 5,

    /// 4096-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r4096"))]
    R4096 = 6,

    /// 8192-bit non-arithmetics register
    #[cfg_attr(feature = "std", display("r8192"))]
    R8192 = 7,
}

impl RegR {
    /// Returns bit size, if defined, for the register.
    ///
    /// Bit size is undefined for [`RegA::AP`] register
    pub fn bits(self) -> Option<u16> {
        match self {
            RegR::R128 => Some(128),
            RegR::R160 => Some(160),
            RegR::R256 => Some(256),
            RegR::R512 => Some(512),
            RegR::R1024 => Some(1024),
            RegR::R2048 => Some(2048),
            RegR::R4096 => Some(4096),
            RegR::R8192 => Some(8192),
        }
    }
}

impl From<&RegR> for u3 {
    fn from(regr: &RegR) -> Self {
        u3::with(*regr as u8)
    }
}

impl From<RegR> for u3 {
    fn from(regr: RegR) -> Self {
        u3::with(regr as u8)
    }
}

impl From<u3> for RegR {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegR::R128.into() => RegR::R128,
            v if v == RegR::R160.into() => RegR::R160,
            v if v == RegR::R256.into() => RegR::R256,
            v if v == RegR::R512.into() => RegR::R512,
            v if v == RegR::R1024.into() => RegR::R1024,
            v if v == RegR::R2048.into() => RegR::R2048,
            v if v == RegR::R4096.into() => RegR::R4096,
            v if v == RegR::R8192.into() => RegR::R8192,
            _ => unreachable!(),
        }
    }
}

/// All non-string registers directly accessible by AluVM instructions,
/// consisting of `a` and `r` sets of registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display, From), display(inner))]
pub enum Reg {
    /// Arithmetic registers (`a` registers)
    #[cfg_attr(feature = "std", from)]
    A(RegA),

    /// Non-arithmetic (generic) registers (`r` registers)
    #[cfg_attr(feature = "std", from)]
    R(RegR),
}

impl Reg {
    /// Returns bit size, if defined, for the register.
    ///
    /// Bit size is undefined for [`RegA::AP`] register
    pub fn bits(self) -> Option<u16> {
        match self {
            Reg::A(a) => a.bits(),
            Reg::R(r) => r.bits(),
        }
    }

    /// Returns inner A-register type, if any
    pub fn reg_a(self) -> Option<RegA> {
        match self {
            Reg::A(a) => Some(a),
            Reg::R(_) => None,
        }
    }

    /// Returns inner R-register type, if any
    pub fn reg_r(self) -> Option<RegR> {
        match self {
            Reg::A(_) => None,
            Reg::R(r) => Some(r),
        }
    }
}

impl From<&Reg> for u4 {
    fn from(reg: &Reg) -> Self {
        u4::from(*reg)
    }
}

impl From<Reg> for u4 {
    fn from(reg: Reg) -> Self {
        match reg {
            Reg::A(a) => u4::with(u3::from(a).as_u8()),
            Reg::R(r) => u4::with(u3::from(r).as_u8() + 8),
        }
    }
}

impl From<u4> for Reg {
    fn from(val: u4) -> Self {
        match val.as_u8() {
            0..=7 => Reg::A(RegA::from(u3::with(val.as_u8()))),
            _ => Reg::R(RegR::from(u3::with(val.as_u8() + 8))),
        }
    }
}

/// Block of registers, either arithmetic or non-arithmetic
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum RegBlock {
    /// Arithmetic registers (`a` registers)
    #[cfg_attr(feature = "std", display("a"))]
    A,

    /// Non-arithmetic (generic) registers (`r` registers)
    #[cfg_attr(feature = "std", display("r"))]
    R,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Registers {
    /// Arbitrary-precision arithmetics registers
    pub(crate) ap: [Option<Value>; 32],

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
    pub(crate) s16: BTreeMap<u8, Vec<u8>>,

    /// Control flow register which stores result of equality and other types
    /// of boolean checks. Initialized with `true`
    pub(crate) st0: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is
    /// limited by 2^16 per script.
    cy0: u16,

    /// Call stack. Maximal size is `u16::MAX` (limited by `cy0` mechanics and
    /// `cp0`)
    cs0: Box<[LibSite; u16::MAX as usize]>,

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
            cs0: Box::new([LibSite::default(); u16::MAX as usize]),
            s16: Default::default(),
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

    pub fn all(&self, reg: impl Into<Reg>) -> [RegVal; 32] {
        let mut res = [RegVal::none(); 32];
        match reg.into() {
            Reg::A(a) => match a {
                RegA::AP => self
                    .ap
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A8 => self
                    .a8
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A16 => self
                    .a16
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A32 => self
                    .a32
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A64 => self
                    .a64
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A128 => self
                    .a128
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A256 => self
                    .a256
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegA::A512 => self
                    .a512
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
            },

            Reg::R(r) => match r {
                RegR::R128 => self
                    .r128
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R160 => self
                    .r160
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R256 => self
                    .r256
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R512 => self
                    .r512
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R1024 => self
                    .r1024
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R2048 => self
                    .r2048
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R4096 => self
                    .r4096
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
                RegR::R8192 => self
                    .r8192
                    .iter()
                    .map(RegVal::from)
                    .enumerate()
                    .for_each(|(idx, val)| res[idx] = val),
            },
        }
        res
    }

    pub fn get(&self, reg: impl Into<Reg>, index: impl Into<Reg32>) -> RegVal {
        let index = index.into() as usize;
        match reg.into() {
            Reg::A(a) => match a {
                RegA::AP => self.ap[index].map(Value::from),
                RegA::A8 => self.a8[index].map(Value::from),
                RegA::A16 => self.a16[index].map(Value::from),
                RegA::A32 => self.a32[index].map(Value::from),
                RegA::A64 => self.a64[index].map(Value::from),
                RegA::A128 => self.a128[index].map(Value::from),
                RegA::A256 => self.a256[index].map(Value::from),
                RegA::A512 => self.a512[index].map(Value::from),
            },

            Reg::R(r) => match r {
                RegR::R128 => self.r128[index].map(Value::from),
                RegR::R160 => self.r160[index].map(Value::from),
                RegR::R256 => self.r256[index].map(Value::from),
                RegR::R512 => self.r512[index].map(Value::from),
                RegR::R1024 => self.r1024[index].map(Value::from),
                RegR::R2048 => self.r2048[index].map(Value::from),
                RegR::R4096 => self.r4096[index].map(Value::from),
                RegR::R8192 => self.r8192[index].map(Value::from),
            },
        }
        .into()
    }

    pub fn set(&mut self, reg: impl Into<Reg>, index: impl Into<Reg32>, value: impl Into<RegVal>) {
        let index = index.into() as usize;
        let value: Option<Value> = value.into().into();
        match reg.into() {
            Reg::A(a) => match a {
                RegA::AP => self.ap[index] = value.map(Value::into),
                RegA::A8 => self.a8[index] = value.map(Value::into),
                RegA::A16 => self.a16[index] = value.map(Value::into),
                RegA::A32 => self.a32[index] = value.map(Value::into),
                RegA::A64 => self.a64[index] = value.map(Value::into),
                RegA::A128 => self.a128[index] = value.map(Value::into),
                RegA::A256 => self.a256[index] = value.map(Value::into),
                RegA::A512 => self.a512[index] = value.map(Value::into),
            },
            Reg::R(r) => match r {
                RegR::R128 => self.r128[index] = value.map(Value::into),
                RegR::R160 => self.r160[index] = value.map(Value::into),
                RegR::R256 => self.r256[index] = value.map(Value::into),
                RegR::R512 => self.r512[index] = value.map(Value::into),
                RegR::R1024 => self.r1024[index] = value.map(Value::into),
                RegR::R2048 => self.r2048[index] = value.map(Value::into),
                RegR::R4096 => self.r4096[index] = value.map(Value::into),
                RegR::R8192 => self.r8192[index] = value.map(Value::into),
            },
        }
    }

    pub fn set_if(&mut self, reg: impl Into<Reg>, index: impl Into<Reg32>, value: Value) {
        let index = index.into();
        let reg = reg.into();
        if self.get(reg, index).deref().is_none() {
            return;
        }
        let index = index as usize;
        match reg {
            Reg::A(a) => match a {
                RegA::AP => self.ap[index] = Some(value.into()),
                RegA::A8 => self.a8[index] = Some(value.into()),
                RegA::A16 => self.a16[index] = Some(value.into()),
                RegA::A32 => self.a32[index] = Some(value.into()),
                RegA::A64 => self.a64[index] = Some(value.into()),
                RegA::A128 => self.a128[index] = Some(value.into()),
                RegA::A256 => self.a256[index] = Some(value.into()),
                RegA::A512 => self.a512[index] = Some(value.into()),
            },
            Reg::R(r) => match r {
                RegR::R128 => self.r128[index] = Some(value.into()),
                RegR::R160 => self.r160[index] = Some(value.into()),
                RegR::R256 => self.r256[index] = Some(value.into()),
                RegR::R512 => self.r512[index] = Some(value.into()),
                RegR::R1024 => self.r1024[index] = Some(value.into()),
                RegR::R2048 => self.r2048[index] = Some(value.into()),
                RegR::R4096 => self.r4096[index] = Some(value.into()),
                RegR::R8192 => self.r8192[index] = Some(value.into()),
            },
        }
    }

    #[inline]
    pub fn op(
        &mut self,
        reg: RegA,
        src1: impl Into<Reg32>,
        src2: impl Into<Reg32>,
        dst: impl Into<Reg32>,
        op: fn(Value, Value) -> Value,
    ) {
        let reg_val = match (*self.get(reg, src1), *self.get(reg, src2)) {
            (None, None) | (None, Some(_)) | (Some(_), None) => RegVal::none(),
            (Some(val1), Some(val2)) => op(val1, val2).into(),
        };
        self.set(reg, dst, reg_val);
    }

    #[inline]
    pub fn op_ap1(
        &mut self,
        reg: RegA,
        index: impl Into<Reg32>,
        ap: bool,
        dst: impl Into<Reg32>,
        op: impl Fn(Value) -> Option<Value>,
    ) {
        let reg_val = self
            .get(reg, index)
            .and_then(op)
            .map(RegVal::from)
            .unwrap_or_default();
        self.set(if ap { RegA::AP } else { reg }, dst, reg_val);
    }

    #[inline]
    pub fn op_ap2(
        &mut self,
        reg: RegA,
        src1: impl Into<Reg32>,
        src2: impl Into<Reg32>,
        ap: bool,
        dst: impl Into<Reg32>,
        op: fn(Value, Value) -> Option<Value>,
    ) {
        let reg_val = match (*self.get(reg, src1), *self.get(reg, src2)) {
            (None, None) | (None, Some(_)) | (Some(_), None) => RegVal::none(),
            (Some(val1), Some(val2)) => op(val1, val2).into(),
        };
        self.set(if ap { RegA::AP } else { reg }, dst, reg_val);
    }

    pub fn status(&self) -> bool {
        return self.st0;
    }
}
