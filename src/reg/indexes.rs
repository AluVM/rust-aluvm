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

use amplify_num::{u3, u4, u5};

/// All possible register indexes for `a` and `r` register sets
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum Reg32 {
    /// Register with index `[1]`
    #[display("[1]")]
    Reg1 = 0,

    /// Register with index `[2]`
    #[display("[2]")]
    Reg2 = 1,

    /// Register with index `[3]`
    #[display("[3]")]
    Reg3 = 2,

    /// Register with index `[4]`
    #[display("[4]")]
    Reg4 = 3,

    /// Register with index `[5]`
    #[display("[5]")]
    Reg5 = 4,

    /// Register with index `[6]`
    #[display("[6]")]
    Reg6 = 5,

    /// Register with index `[7]`
    #[display("[7]")]
    Reg7 = 6,

    /// Register with index `[8]`
    #[display("[8]")]
    Reg8 = 7,

    /// Register with index `[9]`
    #[display("[9]")]
    Reg9 = 8,

    /// Register with index `[10]`
    #[display("[10]")]
    Reg10 = 9,

    /// Register with index `[11]`
    #[display("[11]")]
    Reg11 = 10,

    /// Register with index `[12]`
    #[display("[12]")]
    Reg12 = 11,

    /// Register with index `[13]`
    #[display("[13]")]
    Reg13 = 12,

    /// Register with index `[14]`
    #[display("[14]")]
    Reg14 = 13,

    /// Register with index `[15]`
    #[display("[15]")]
    Reg15 = 14,

    /// Register with index `[16]`
    #[display("[16]")]
    Reg16 = 15,

    /// Register with index `[17]`
    #[display("[17]")]
    Reg17 = 16,

    /// Register with index `[18]`
    #[display("[18]")]
    Reg18 = 17,

    /// Register with index `[19]`
    #[display("[19]")]
    Reg19 = 18,

    /// Register with index `[20]`
    #[display("[10]")]
    Reg20 = 19,

    /// Register with index `[21]`
    #[display("[21]")]
    Reg21 = 20,

    /// Register with index `[22]`
    #[display("[22]")]
    Reg22 = 21,

    /// Register with index `[23]`
    #[display("[23]")]
    Reg23 = 22,

    /// Register with index `[24]`
    #[display("[24]")]
    Reg24 = 23,

    /// Register with index `[25]`
    #[display("[25]")]
    Reg25 = 24,

    /// Register with index `[26]`
    #[display("[26]")]
    Reg26 = 25,

    /// Register with index `[27]`
    #[display("[27]")]
    Reg27 = 26,

    /// Register with index `[28]`
    #[display("[28]")]
    Reg28 = 27,

    /// Register with index `[29]`
    #[display("[29]")]
    Reg29 = 28,

    /// Register with index `[30]`
    #[display("[30]")]
    Reg30 = 29,

    /// Register with index `[31]`
    #[display("[31]")]
    Reg31 = 30,

    /// Register with index `[32]`
    #[display("[32]")]
    Reg32 = 31,
}

impl Reg32 {
    /// Returns `usize` representation of the register index
    pub fn to_usize(self) -> usize { self as u8 as usize }
}

impl Default for Reg32 {
    fn default() -> Self { Reg32::Reg1 }
}

impl From<&Reg32> for u5 {
    fn from(reg32: &Reg32) -> Self { u5::with(*reg32 as u8) }
}

impl From<Reg32> for u5 {
    fn from(reg32: Reg32) -> Self { u5::with(reg32 as u8) }
}

impl From<&Reg32> for u8 {
    fn from(reg32: &Reg32) -> Self { *reg32 as u8 }
}

impl From<Reg32> for u8 {
    fn from(reg32: Reg32) -> Self { reg32 as u8 }
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum Reg16 {
    /// Register with index `[1]`
    #[display("[1]")]
    Reg1 = 0,

    /// Register with index `[2]`
    #[display("[2]")]
    Reg2 = 1,

    /// Register with index `[3]`
    #[display("[3]")]
    Reg3 = 2,

    /// Register with index `[4]`
    #[display("[4]")]
    Reg4 = 3,

    /// Register with index `[5]`
    #[display("[5]")]
    Reg5 = 4,

    /// Register with index `[6]`
    #[display("[6]")]
    Reg6 = 5,

    /// Register with index `[7]`
    #[display("[7]")]
    Reg7 = 6,

    /// Register with index `[8]`
    #[display("[8]")]
    Reg8 = 7,

    /// Register with index `[9]`
    #[display("[9]")]
    Reg9 = 8,

    /// Register with index `[10]`
    #[display("[10]")]
    Reg10 = 9,

    /// Register with index `[11]`
    #[display("[11]")]
    Reg11 = 10,

    /// Register with index `[12]`
    #[display("[12]")]
    Reg12 = 11,

    /// Register with index `[13]`
    #[display("[13]")]
    Reg13 = 12,

    /// Register with index `[14]`
    #[display("[14]")]
    Reg14 = 13,

    /// Register with index `[15]`
    #[display("[15]")]
    Reg15 = 14,

    /// Register with index `[16]`
    #[display("[16]")]
    Reg16 = 15,
}

impl Default for Reg16 {
    fn default() -> Self { Reg16::Reg1 }
}

impl From<&Reg16> for u4 {
    fn from(reg16: &Reg16) -> Self { u4::with(*reg16 as u8) }
}

impl From<Reg16> for u4 {
    fn from(reg16: Reg16) -> Self { u4::with(reg16 as u8) }
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
    fn from(reg16: Reg16) -> Self { u5::with(reg16 as u8).into() }
}

/// Short version of register indexes for `a` and `r` register sets covering
/// initial 8 registers only
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(u8)]
pub enum Reg8 {
    /// Register with index `[1]`
    #[display("[1]")]
    Reg1 = 0,

    /// Register with index `[2]`
    #[display("[2]")]
    Reg2 = 1,

    /// Register with index `[3]`
    #[display("[3]")]
    Reg3 = 2,

    /// Register with index `[4]`
    #[display("[4]")]
    Reg4 = 3,

    /// Register with index `[5]`
    #[display("[5]")]
    Reg5 = 4,

    /// Register with index `[6]`
    #[display("[6]")]
    Reg6 = 5,

    /// Register with index `[7]`
    #[display("[7]")]
    Reg7 = 6,

    /// Register with index `[8]`
    #[display("[8]")]
    Reg8 = 7,
}

impl Default for Reg8 {
    fn default() -> Self { Reg8::Reg1 }
}

impl From<&Reg8> for u3 {
    fn from(reg8: &Reg8) -> Self { u3::with(*reg8 as u8) }
}

impl From<Reg8> for u3 {
    fn from(reg8: Reg8) -> Self { u3::with(reg8 as u8) }
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
    fn from(reg8: Reg8) -> Self { u5::with(reg8 as u8).into() }
}
