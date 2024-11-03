// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 UBIDECO Labs,
//     Laboratories for Distributed and Cognitive Computing, Switzerland.
//     All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::core::SiteId;
use crate::{Core, LIB_NAME_ALUVM};

const M31: u128 = (1 << 31u128) - 1;
const F1137119: u128 = 1 + 11 * 37 * (1 << 119u128);
const F1289: u128 = u128::MAX - 8; // it should be 9, but `u128::MAX` is 2^128-1 and not 2^128

/// Finite field orders.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_ALUVM, tags = custom)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Fq {
    #[display("M31", alt = "2^31-1")]
    #[strict_type(tag = 31, dumb)]
    M31, // 2^31-1

    #[display("F1137119", alt = "1+11*37*2^119")]
    #[strict_type(tag = 119)]
    F1137119,

    #[display("F1289", alt = "2^128-9")]
    #[strict_type(tag = 128)]
    F1289,

    #[display("{0:X}#h")]
    #[strict_type(tag = 0xFF)]
    Other(u128),
}

impl From<Fq> for u128 {
    fn from(fq: Fq) -> Self { fq.to_u128() }
}

impl Fq {
    pub fn to_u128(self) -> u128 {
        match self {
            Fq::M31 => M31,
            Fq::F1137119 => F1137119,
            Fq::F1289 => F1289,
            Fq::Other(val) => val,
        }
    }
}

/// Microcode for finite field arithmetics.
impl<Id: SiteId> Core<Id> {
    pub fn fq(&self) -> Fq { self.fq }
    pub fn fq_u128(&self) -> u128 { self.fq.to_u128() }

    #[inline]
    pub fn add_mod(&mut self, a: u128, b: u128) -> Option<u128> {
        let order = self.fq.to_u128();
        if a >= order || b >= order {
            return None;
        }

        let (mut res, overflow) = a.overflowing_add(b);
        if overflow {
            res += u128::MAX - order;
        }
        res %= order;

        self.set_co(overflow);
        Some(res)
    }

    #[inline]
    pub fn mul_mod(&mut self, a: u128, b: u128) -> Option<u128> {
        let order = self.fq.to_u128();
        if a >= order || b >= order {
            return None;
        }

        let (res, overflow) = self.mul_mod_int(a, b);

        self.set_co(overflow);
        Some(res)
    }

    fn mul_mod_int(&mut self, a: u128, b: u128) -> (u128, bool) {
        let order = self.fq.to_u128();
        let (mut res, overflow) = a.overflowing_mul(b);
        if overflow {
            let rem = u128::MAX - order;
            res = self.mul_mod_int(res, rem).0;
        }
        (res % order, overflow)
    }

    #[inline]
    pub fn neg_mod(&self, a: u128) -> Option<u128> {
        let order = self.fq.to_u128();
        if a >= order {
            return None;
        }
        Some(order - a)
    }
}
