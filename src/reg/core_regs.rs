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

use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use amplify_num::{u256, u512};
use half::bf16;
use rustc_apfloat::ieee;

use super::{Reg32, RegA, RegAFR, RegF, RegR};
use crate::data::{ByteStr, MaybeNumber, Number};
use crate::libs::LibSite;

/// Structure keeping state of all registers in a single microprosessor/VM core
#[derive(Clone, Debug)]
pub struct CoreRegs {
    // Arithmetic integer registers:
    pub(crate) a8: [Option<u8>; 32],
    pub(crate) a16: [Option<u16>; 32],
    pub(crate) a32: [Option<u32>; 32],
    pub(crate) a64: [Option<u64>; 32],
    pub(crate) a128: [Option<u128>; 32],
    pub(crate) a256: [Option<u256>; 32],
    pub(crate) a512: [Option<u512>; 32],
    pub(crate) a1024: [Option<Number>; 32],

    // Arithmetic float registers
    pub(crate) f16b: [Option<bf16>; 32],
    pub(crate) f16: [Option<ieee::Half>; 32],
    pub(crate) f32: [Option<ieee::Single>; 32],
    pub(crate) f64: [Option<ieee::Double>; 32],
    pub(crate) f80: [Option<ieee::X87DoubleExtended>; 32],
    pub(crate) f128: [Option<ieee::Quad>; 32],
    // TODO(#4) Replace with `ieee::Oct` once it will be implemented
    pub(crate) f256: [Option<u256>; 32],
    // TODO(#5) Implement tapered floating point type
    pub(crate) f512: [Option<u512>; 32],

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
    pub(crate) s16: BTreeMap<u8, ByteStr>,

    /// Control flow register which stores result of equality, comparison, boolean check and
    /// overflowing operations. Initialized with `true`.
    pub(crate) st0: bool,

    /// Counts number of jumps (possible cycles). The number of jumps is limited by 2^16 per
    /// script.
    cy0: u16,

    /// Call stack. Maximal size is `u16::MAX` (limited by `cy0` mechanics and `cp0`)
    cs0: Box<[LibSite; u16::MAX as usize]>,

    /// Defines "top" of the call stack
    cp0: u16,

    /// Secp256k1 context object (used by [`crate::isa::Secp256k1Op`] instructions)
    #[cfg(feature = "secp256k1")]
    pub(crate) secp: secp256k1::Secp256k1<secp256k1::All>,
}

impl Default for CoreRegs {
    #[inline]
    fn default() -> Self {
        CoreRegs {
            a8: Default::default(),
            a16: Default::default(),
            a32: Default::default(),
            a64: Default::default(),
            a128: Default::default(),
            a256: Default::default(),
            a512: Default::default(),
            a1024: Default::default(),

            f16b: Default::default(),
            f16: Default::default(),
            f32: Default::default(),
            f64: Default::default(),
            f80: Default::default(),
            f128: Default::default(),
            f256: Default::default(),
            f512: Default::default(),

            r128: Default::default(),
            r160: Default::default(),
            r256: Default::default(),
            r512: Default::default(),
            r1024: Default::default(),
            r2048: Default::default(),
            r4096: Default::default(),
            r8192: Default::default(),

            s16: Default::default(),

            st0: true,
            // TODO(#2) Introduce `ca0` register
            cy0: 0,
            // TODO(#13) Convert into short library references
            cs0: Box::new([LibSite::default(); u16::MAX as usize]),
            cp0: 0,

            // TODO(#14) Make it a part of the vm
            #[cfg(feature = "secp256k1")]
            secp: secp256k1::Secp256k1::new(),
        }
    }
}

impl CoreRegs {
    /// Initializes register state. Sets `st0` to `true`, counters to zero, call stack to empty and
    /// the rest of registers to `None` value.
    ///
    /// Performs exactly the same as [`CoreRegs::default()`].
    #[inline]
    pub fn new() -> CoreRegs { CoreRegs::default() }

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
            .map(|_| {
                self.cs0[self.cp0 as usize] = site;
            })
            .and_then(|_| {
                self.cp0
                    .checked_add(1)
                    .ok_or_else(|| {
                        self.st0 = false;
                    })
                    .map(|_| ())
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

    /// Retrieves register value
    pub fn get(&self, reg: impl Into<RegAFR>, index: impl Into<Reg32>) -> MaybeNumber {
        let index = index.into() as usize;
        match reg.into() {
            RegAFR::A(a) => match a {
                RegA::A8 => self.a8[index].map(Number::from),
                RegA::A16 => self.a16[index].map(Number::from),
                RegA::A32 => self.a32[index].map(Number::from),
                RegA::A64 => self.a64[index].map(Number::from),
                RegA::A128 => self.a128[index].map(Number::from),
                RegA::A256 => self.a256[index].map(Number::from),
                RegA::A512 => self.a512[index].map(Number::from),
                RegA::A1024 => self.a1024[index].map(Number::from),
            },

            RegAFR::R(r) => match r {
                RegR::R128 => self.r128[index].map(Number::from),
                RegR::R160 => self.r160[index].map(Number::from),
                RegR::R256 => self.r256[index].map(Number::from),
                RegR::R512 => self.r512[index].map(Number::from),
                RegR::R1024 => self.r1024[index].map(Number::from),
                RegR::R2048 => self.r2048[index].map(Number::from),
                RegR::R4096 => self.r4096[index].map(Number::from),
                RegR::R8192 => self.r8192[index].map(Number::from),
            },

            RegAFR::F(f) => match f {
                RegF::F16B => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F16 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F32 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F64 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F80 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F128 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F256 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
                RegF::F512 => self.r128[index].map(|slice| Number::with_reg(slice, f)),
            },
        }
        .into()
    }

    /// Returns value from one of `S`-registers
    #[inline]
    pub fn get_s(&self, index: impl Into<u8>) -> Option<&ByteStr> { self.s16.get(&index.into()) }

    /// Returns value from two registers only if both of them contain a value; otherwise returns
    /// `None`.
    #[inline]
    pub fn get_both(
        &self,
        reg1: impl Into<RegAFR>,
        idx1: impl Into<Reg32>,
        reg2: impl Into<RegAFR>,
        idx2: impl Into<Reg32>,
    ) -> Option<(Number, Number)> {
        self.get(reg1, idx1).and_then(|val1| self.get(reg2, idx2).map(|val2| (val1, val2)))
    }

    /// Returns value from two string (`S`) registers only if both of them contain a value;
    /// otherwise returns `None`.
    #[inline]
    pub fn get_both_s(&self, idx1: u8, idx2: u8) -> Option<(&ByteStr, &ByteStr)> {
        self.get_s(idx1).and_then(|val1| self.get_s(idx2).map(|val2| (val1, val2)))
    }

    /// Assigns the provided value to the register bit-wise. Silently discards most significant bits
    /// until the value fits register bit size.
    ///
    /// Returns `true` if the value was not `None`
    pub fn set(
        &mut self,
        reg: impl Into<RegAFR>,
        index: impl Into<Reg32>,
        value: impl Into<MaybeNumber>,
    ) -> bool {
        let index = index.into() as usize;
        let value: Option<Number> = value.into().into();
        match reg.into() {
            RegAFR::A(a) => match a {
                RegA::A1024 => self.a1024[index] = value.map(Number::into),
                RegA::A8 => self.a8[index] = value.map(Number::into),
                RegA::A16 => self.a16[index] = value.map(Number::into),
                RegA::A32 => self.a32[index] = value.map(Number::into),
                RegA::A64 => self.a64[index] = value.map(Number::into),
                RegA::A128 => self.a128[index] = value.map(Number::into),
                RegA::A256 => self.a256[index] = value.map(Number::into),
                RegA::A512 => self.a512[index] = value.map(Number::into),
            },
            RegAFR::R(r) => match r {
                RegR::R128 => self.r128[index] = value.map(Number::into),
                RegR::R160 => self.r160[index] = value.map(Number::into),
                RegR::R256 => self.r256[index] = value.map(Number::into),
                RegR::R512 => self.r512[index] = value.map(Number::into),
                RegR::R1024 => self.r1024[index] = value.map(Number::into),
                RegR::R2048 => self.r2048[index] = value.map(Number::into),
                RegR::R4096 => self.r4096[index] = value.map(Number::into),
                RegR::R8192 => self.r8192[index] = value.map(Number::into),
            },
            RegAFR::F(f) => match f {
                RegF::F16B => self.f16b[index] = value.map(Number::into),
                RegF::F16 => self.f16b[index] = value.map(Number::into),
                RegF::F32 => self.f16b[index] = value.map(Number::into),
                RegF::F64 => self.f16b[index] = value.map(Number::into),
                RegF::F80 => self.f16b[index] = value.map(Number::into),
                RegF::F128 => self.f16b[index] = value.map(Number::into),
                RegF::F256 => self.f16b[index] = value.map(Number::into),
                RegF::F512 => self.f16b[index] = value.map(Number::into),
            },
        }
        value.is_some()
    }

    /// Assigns the provided value to the register bit-wise if the register is not initialized.
    /// Silently discards most significant bits until the value fits register bit size.
    ///
    /// Returns `true` if the value was not `None` and the register was initialized.
    #[inline]
    pub fn set_if(
        &mut self,
        reg: impl Into<RegAFR>,
        index: impl Into<Reg32>,
        value: impl Into<MaybeNumber>,
    ) -> bool {
        let reg = reg.into();
        let index = index.into();
        if self.get(reg, index).is_none() {
            self.set(reg, index, value)
        } else {
            false
        }
    }

    /// Executes provided operation (as callback function) if and only if all the provided registers
    /// contain a value (initialized). Otherwise, sets destination to `None` and does not calls the
    /// callback.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn op(
        &mut self,
        reg1: impl Into<RegAFR>,
        src1: impl Into<Reg32>,
        reg2: impl Into<RegAFR>,
        src2: impl Into<Reg32>,
        reg3: impl Into<RegAFR>,
        dst: impl Into<Reg32>,
        op: fn(Number, Number) -> Number,
    ) {
        let reg_val = match (*self.get(reg1.into(), src1), *self.get(reg2.into(), src2)) {
            (None, None) | (None, Some(_)) | (Some(_), None) => MaybeNumber::none(),
            (Some(val1), Some(val2)) => op(val1, val2).into(),
        };
        self.set(reg3.into(), dst, reg_val);
    }

    /// Returns vale of `st0` register
    #[inline]
    pub fn status(&self) -> bool { self.st0 }
}
