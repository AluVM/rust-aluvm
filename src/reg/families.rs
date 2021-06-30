// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://github.com/internet2-org/aluvm-spec>
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::{u1, u3, u4};

use crate::data as number;

/// Common set of methods handled by different sets and families of VM registers
pub trait RegisterFamily {
    /// Register bit dimension
    #[inline]
    fn bits(&self) -> u16 { self.bytes() * 8 }

    /// Size of the register value in bytes
    fn bytes(&self) -> u16;

    /// Returns register layout
    fn layout(&self) -> number::Layout;
}

/// Enumeration of integer arithmetic registers (`A`-registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum RegA {
    /// 8-bit arithmetics register
    #[display("a8")]
    A8 = 0,

    /// 16-bit arithmetics register
    #[display("a16")]
    A16 = 1,

    /// 32-bit arithmetics register
    #[display("a32")]
    A32 = 2,

    /// 64-bit arithmetics register
    #[display("a64")]
    A64 = 3,

    /// 128-bit arithmetics register
    #[display("a128")]
    A128 = 4,

    /// 256-bit arithmetics register
    #[display("a256")]
    A256 = 5,

    /// 512-bit arithmetics register
    #[display("a512")]
    A512 = 6,

    /// 1024-bit arithmetics register
    #[display("a1024")]
    A1024 = 7,
}

impl RegisterFamily for RegA {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegA::A8 => 1,
            RegA::A16 => 2,
            RegA::A32 => 4,
            RegA::A64 => 8,
            RegA::A128 => 16,
            RegA::A256 => 32,
            RegA::A512 => 64,
            RegA::A1024 => 128,
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout { number::Layout::unsigned(self.bytes()) }
}

impl RegA {
    /// Constructs [`RegA`] object for a provided requirement for register bit size
    pub fn with(bits: u16) -> Option<Self> {
        Some(match bits {
            8 => RegA::A8,
            16 => RegA::A16,
            32 => RegA::A32,
            64 => RegA::A64,
            128 => RegA::A128,
            256 => RegA::A256,
            512 => RegA::A512,
            1024 => RegA::A1024,
            _ => return None,
        })
    }

    /// Returns integer layout [`number::IntLayout`] specific for this register
    #[inline]
    pub fn int_layout(self) -> number::IntLayout { number::IntLayout::unsigned(self.bytes()) }
}

impl From<&RegA> for u3 {
    fn from(rega: &RegA) -> Self { u3::with(*rega as u8) }
}

impl From<RegA> for u3 {
    fn from(rega: RegA) -> Self { u3::with(rega as u8) }
}

impl From<u3> for RegA {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegA::A8.into() => RegA::A8,
            v if v == RegA::A16.into() => RegA::A16,
            v if v == RegA::A32.into() => RegA::A32,
            v if v == RegA::A64.into() => RegA::A64,
            v if v == RegA::A128.into() => RegA::A128,
            v if v == RegA::A256.into() => RegA::A256,
            v if v == RegA::A512.into() => RegA::A512,
            v if v == RegA::A1024.into() => RegA::A1024,
            _ => unreachable!(),
        }
    }
}

impl From<RegA2> for RegA {
    #[inline]
    fn from(reg: RegA2) -> Self {
        match reg {
            RegA2::A8 => RegA::A8,
            RegA2::A16 => RegA::A16,
        }
    }
}

impl From<&RegA2> for RegA {
    #[inline]
    fn from(reg: &RegA2) -> Self {
        match reg {
            RegA2::A8 => RegA::A8,
            RegA2::A16 => RegA::A16,
        }
    }
}

/// Enumeration of integer arithmetic registers suited for string addresses (`a8` and `a16`
/// registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum RegA2 {
    /// 8-bit arithmetics register
    #[display("a8")]
    A8 = 0,

    /// 16-bit arithmetics register
    #[display("a16")]
    A16 = 1,
}

impl RegisterFamily for RegA2 {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegA2::A8 => 1,
            RegA2::A16 => 2,
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout { number::Layout::unsigned(self.bytes()) }
}

impl RegA2 {
    /// Constructs [`RegA2`] object for a provided requirement for register bit size
    pub fn with(bits: u16) -> Option<Self> {
        Some(match bits {
            8 => RegA2::A8,
            16 => RegA2::A16,
            _ => return None,
        })
    }
}

impl From<&RegA2> for u1 {
    fn from(rega: &RegA2) -> Self { u1::with(*rega as u8) }
}

impl From<RegA2> for u1 {
    fn from(rega: RegA2) -> Self { u1::with(rega as u8) }
}

impl From<u1> for RegA2 {
    fn from(val: u1) -> Self {
        match val {
            v if v == RegA2::A8.into() => RegA2::A8,
            v if v == RegA2::A16.into() => RegA2::A16,
            _ => unreachable!(),
        }
    }
}

/// Enumeration of float arithmetic registers (`F`-registers)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum RegF {
    /// 16-bit bfloat16 format used in machine learning
    #[display("f16b")]
    F16B = 0,

    /// 16-bit IEEE-754 binary16 half-precision
    #[display("f16")]
    F16 = 1,

    /// 32-bit IEEE-754 binary32 single-precision
    #[display("f32")]
    F32 = 2,

    /// 64-bit IEEE-754 binary64 double-precision
    #[display("f64")]
    F64 = 3,

    /// 80-bit IEEE-754 extended precision
    #[display("f80")]
    F80 = 4,

    /// 128-bit IEEE-754 binary128 quadruple precision
    #[display("f128")]
    F128 = 5,

    /// 256-bit IEEE-754 binary256 octuple precision
    #[display("f256")]
    F256 = 6,

    /// 512-bit tapered floating point
    #[display("f512")]
    F512 = 7,
}

impl RegisterFamily for RegF {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegF::F16B => 2,
            RegF::F16 => 2,
            RegF::F32 => 4,
            RegF::F64 => 8,
            RegF::F80 => 10,
            RegF::F128 => 16,
            RegF::F256 => 32,
            RegF::F512 => 64,
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout {
        let fl = match self {
            RegF::F16B => number::FloatLayout::BFloat16,
            RegF::F16 => number::FloatLayout::IeeeHalf,
            RegF::F32 => number::FloatLayout::IeeeSingle,
            RegF::F64 => number::FloatLayout::IeeeDouble,
            RegF::F80 => number::FloatLayout::X87DoubleExt,
            RegF::F128 => number::FloatLayout::IeeeQuad,
            RegF::F256 => number::FloatLayout::IeeeOct,
            RegF::F512 => number::FloatLayout::FloatTapered,
        };
        number::Layout::float(fl)
    }
}

impl RegF {
    /// Constructs [`RegF`] object for a provided requirement for register bit size
    pub fn with(bits: u16, use_bfloat16: bool) -> Option<Self> {
        Some(match bits {
            16 => {
                if use_bfloat16 {
                    RegF::F16B
                } else {
                    RegF::F16
                }
            }
            32 => RegF::F32,
            64 => RegF::F64,
            80 => RegF::F80,
            128 => RegF::F128,
            256 => RegF::F256,
            512 => RegF::F512,
            _ => return None,
        })
    }
}

impl From<&RegF> for u3 {
    fn from(regf: &RegF) -> Self { u3::with(*regf as u8) }
}

impl From<RegF> for u3 {
    fn from(regf: RegF) -> Self { u3::with(regf as u8) }
}

impl From<u3> for RegF {
    fn from(val: u3) -> Self {
        match val {
            v if v == RegF::F16B.into() => RegF::F16B,
            v if v == RegF::F16.into() => RegF::F16,
            v if v == RegF::F32.into() => RegF::F32,
            v if v == RegF::F64.into() => RegF::F64,
            v if v == RegF::F80.into() => RegF::F80,
            v if v == RegF::F128.into() => RegF::F128,
            v if v == RegF::F256.into() => RegF::F256,
            v if v == RegF::F512.into() => RegF::F512,
            _ => unreachable!(),
        }
    }
}

/// Enumeration of the set of general registers (`R`-registers: non-arithmetic registers, mostly
/// used for cryptography)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum RegR {
    /// 128-bit non-arithmetics register
    #[display("r128")]
    R128 = 0,

    /// 160-bit non-arithmetics register
    #[display("r160")]
    R160 = 1,

    /// 256-bit non-arithmetics register
    #[display("r256")]
    R256 = 2,

    /// 512-bit non-arithmetics register
    #[display("r512")]
    R512 = 3,

    /// 1024-bit non-arithmetics register
    #[display("r1024")]
    R1024 = 4,

    /// 2048-bit non-arithmetics register
    #[display("r2048")]
    R2048 = 5,

    /// 4096-bit non-arithmetics register
    #[display("r4096")]
    R4096 = 6,

    /// 8192-bit non-arithmetics register
    #[display("r8192")]
    R8192 = 7,
}

impl RegisterFamily for RegR {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegR::R128 => 16,
            RegR::R160 => 20,
            RegR::R256 => 32,
            RegR::R512 => 64,
            RegR::R1024 => 128,
            RegR::R2048 => 256,
            RegR::R4096 => 512,
            RegR::R8192 => 1024,
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout { number::Layout::unsigned(self.bytes()) }
}

impl RegR {
    /// Constructs [`RegR`] object for a provided requirement for register bit size
    #[inline]
    pub fn with(bits: u16) -> Option<Self> {
        Some(match bits {
            128 => RegR::R128,
            160 => RegR::R160,
            256 => RegR::R256,
            512 => RegR::R512,
            1024 => RegR::R1024,
            2048 => RegR::R2048,
            4096 => RegR::R4096,
            8192 => RegR::R8192,
            _ => return None,
        })
    }
}

impl From<&RegR> for u3 {
    fn from(regr: &RegR) -> Self { u3::with(*regr as u8) }
}

impl From<RegR> for u3 {
    fn from(regr: RegR) -> Self { u3::with(regr as u8) }
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

/// Superset of all registers which value can be represented by a
/// [`crate::data::Number`]/[`crate::data::MaybeNumber`]. The superset includes `A`, `F`, and
/// `R` families of registers.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From)]
#[display(inner)]
pub enum RegAFR {
    /// Arithmetic integer registers (`A` registers)
    #[from]
    A(RegA),

    /// Arithmetic float registers (`F` registers)
    #[from]
    F(RegF),

    /// Non-arithmetic (general) registers (`R` registers)
    #[from]
    R(RegR),
}

impl RegisterFamily for RegAFR {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegAFR::A(a) => a.bytes(),
            RegAFR::F(f) => f.bytes(),
            RegAFR::R(r) => r.bytes(),
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout {
        match self {
            RegAFR::A(a) => a.layout(),
            RegAFR::F(f) => f.layout(),
            RegAFR::R(r) => r.layout(),
        }
    }
}

impl RegAFR {
    /// Returns inner A-register type, if any
    #[inline]
    pub fn reg_a(self) -> Option<RegA> {
        match self {
            RegAFR::A(a) => Some(a),
            _ => None,
        }
    }

    /// Returns inner F-register type, if any
    #[inline]
    pub fn reg_f(self) -> Option<RegF> {
        match self {
            RegAFR::F(f) => Some(f),
            _ => None,
        }
    }

    /// Returns inner R-register type, if any
    #[inline]
    pub fn reg_r(self) -> Option<RegR> {
        match self {
            RegAFR::R(r) => Some(r),
            _ => None,
        }
    }
}

impl From<&RegA> for RegAFR {
    #[inline]
    fn from(reg: &RegA) -> Self { Self::A(*reg) }
}

impl From<&RegF> for RegAFR {
    #[inline]
    fn from(reg: &RegF) -> Self { Self::F(*reg) }
}

impl From<&RegR> for RegAFR {
    #[inline]
    fn from(reg: &RegR) -> Self { Self::R(*reg) }
}

impl From<RegA2> for RegAFR {
    #[inline]
    fn from(reg: RegA2) -> Self { Self::A(reg.into()) }
}

impl From<&RegA2> for RegAFR {
    #[inline]
    fn from(reg: &RegA2) -> Self { Self::A(reg.into()) }
}

impl From<RegAF> for RegAFR {
    #[inline]
    fn from(reg: RegAF) -> Self {
        match reg {
            RegAF::A(a) => Self::A(a),
            RegAF::F(f) => Self::F(f),
        }
    }
}

impl From<&RegAF> for RegAFR {
    #[inline]
    fn from(reg: &RegAF) -> Self {
        match reg {
            RegAF::A(a) => Self::A(*a),
            RegAF::F(f) => Self::F(*f),
        }
    }
}

impl From<RegAR> for RegAFR {
    #[inline]
    fn from(reg: RegAR) -> Self {
        match reg {
            RegAR::A(a) => Self::A(a),
            RegAR::R(r) => Self::R(r),
        }
    }
}

impl From<&RegAR> for RegAFR {
    #[inline]
    fn from(reg: &RegAR) -> Self {
        match reg {
            RegAR::A(a) => Self::A(*a),
            RegAR::R(r) => Self::R(*r),
        }
    }
}

/// Superset of `A` and `F` arithmetic registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From)]
#[display(inner)]
pub enum RegAF {
    /// Arithmetic integer registers (`A` registers)
    #[from]
    A(RegA),

    /// Arithmetic float registers (`F` registers)
    #[from]
    F(RegF),
}

impl RegisterFamily for RegAF {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegAF::A(a) => a.bytes(),
            RegAF::F(f) => f.bytes(),
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout {
        match self {
            RegAF::A(a) => a.layout(),
            RegAF::F(f) => f.layout(),
        }
    }
}

impl RegAF {
    /// Returns inner A-register type, if any
    #[inline]
    pub fn reg_a(self) -> Option<RegA> {
        match self {
            RegAF::A(a) => Some(a),
            RegAF::F(_) => None,
        }
    }

    /// Returns inner F-register type, if any
    #[inline]
    pub fn reg_f(self) -> Option<RegF> {
        match self {
            RegAF::A(_) => None,
            RegAF::F(f) => Some(f),
        }
    }
}

impl From<&RegAF> for u4 {
    fn from(reg: &RegAF) -> Self { u4::from(*reg) }
}

impl From<RegAF> for u4 {
    fn from(reg: RegAF) -> Self {
        match reg {
            RegAF::A(a) => u4::with(u3::from(a).as_u8()),
            RegAF::F(f) => u4::with(u3::from(f).as_u8() + 8),
        }
    }
}

impl From<u4> for RegAF {
    fn from(val: u4) -> Self {
        match val.as_u8() {
            0..=7 => RegAF::A(RegA::from(u3::with(val.as_u8()))),
            _ => RegAF::F(RegF::from(u3::with(val.as_u8() - 8))),
        }
    }
}

impl From<RegA2> for RegAF {
    #[inline]
    fn from(reg: RegA2) -> Self { Self::A(reg.into()) }
}

impl From<&RegA2> for RegAF {
    #[inline]
    fn from(reg: &RegA2) -> Self { Self::A(reg.into()) }
}

/// Superset of `A` and `R` registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From)]
#[display(inner)]
pub enum RegAR {
    /// Arithmetic integer registers (`A` registers)
    #[from]
    A(RegA),

    /// Non-arithmetic (general) registers (`R` registers)
    #[from]
    R(RegR),
}

impl RegisterFamily for RegAR {
    #[inline]
    fn bytes(&self) -> u16 {
        match self {
            RegAR::A(a) => a.bytes(),
            RegAR::R(r) => r.bytes(),
        }
    }

    #[inline]
    fn layout(&self) -> number::Layout {
        match self {
            RegAR::A(a) => a.layout(),
            RegAR::R(r) => r.layout(),
        }
    }
}

impl RegAR {
    /// Constructs register superset from register block and family integer representation
    #[inline]
    pub fn from(block: u1, reg: u3) -> Self {
        match block.as_u8() {
            0 => RegAR::A(reg.into()),
            1 => RegAR::R(reg.into()),
            _ => unreachable!(),
        }
    }

    /// Returns inner A-register type, if any
    #[inline]
    pub fn reg_a(self) -> Option<RegA> {
        match self {
            RegAR::A(a) => Some(a),
            RegAR::R(_) => None,
        }
    }

    /// Returns inner R-register type, if any
    #[inline]
    pub fn reg_r(self) -> Option<RegR> {
        match self {
            RegAR::A(_) => None,
            RegAR::R(r) => Some(r),
        }
    }
}

impl From<&RegAR> for u4 {
    fn from(reg: &RegAR) -> Self { u4::from(*reg) }
}

impl From<RegAR> for u4 {
    fn from(reg: RegAR) -> Self {
        match reg {
            RegAR::A(a) => u4::with(u3::from(a).as_u8()),
            RegAR::R(r) => u4::with(u3::from(r).as_u8() + 8),
        }
    }
}

impl From<u4> for RegAR {
    fn from(val: u4) -> Self {
        match val.as_u8() {
            0..=7 => RegAR::A(RegA::from(u3::with(val.as_u8()))),
            _ => RegAR::R(RegR::from(u3::with(val.as_u8() - 8))),
        }
    }
}

impl From<RegA2> for RegAR {
    #[inline]
    fn from(reg: RegA2) -> Self { Self::A(reg.into()) }
}

impl From<&RegA2> for RegAR {
    #[inline]
    fn from(reg: &RegA2) -> Self { Self::A(reg.into()) }
}

/// Block of registers, either integer arithmetic or non-arithmetic (general) registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum RegBlockAR {
    /// Arithmetic integer registers (`A` registers)
    #[display("a")]
    A,

    /// Non-arithmetic (generic) registers (`R` registers)
    #[display("r")]
    R,
}

impl RegBlockAR {
    /// Converts value into specific register matching the provided bit dimension. If the register
    /// with the given dimension does not exists, returns `None`.
    pub fn into_reg(self, bits: u16) -> Option<RegAR> {
        match self {
            RegBlockAR::A => RegA::with(bits).map(RegAR::A),
            RegBlockAR::R => RegR::with(bits).map(RegAR::R),
        }
    }
}

/// Block of registers, either integer, float arithmetic or non-arithmetic (general) registers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum RegBlockAFR {
    /// Arithmetic integer registers (`A` registers)
    #[display("a")]
    A,

    /// Arithmetic float registers (`F` registers)
    #[display("f")]
    F,

    /// Non-arithmetic (generic) registers (`R` registers)
    #[display("r")]
    R,
}

impl RegBlockAFR {
    /// Converts value into specific register matching the provided bit dimension. If the register
    /// with the given dimension does not exists, returns `None`.
    pub fn into_reg(self, bits: u16) -> Option<RegAFR> {
        match self {
            RegBlockAFR::A => RegA::with(bits).map(RegAFR::A),
            RegBlockAFR::F => RegF::with(bits, false).map(RegAFR::F),
            RegBlockAFR::R => RegR::with(bits).map(RegAFR::R),
        }
    }
}
