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

use core::fmt::{self, Debug, Display, Formatter};
use core::str::FromStr;

use amplify::num::{u4, u5};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
#[repr(i8)]
pub enum Status {
    #[display("ok")]
    Ok = 0,

    #[display("fail")]
    Fail = -1,
}

impl Status {
    pub fn is_ok(self) -> bool { self == Status::Ok }
}

pub trait SiteId: Copy + Ord + Debug + Display + FromStr {}

/// Location inside the instruction sequence which can be executed by the core.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Site<Id: SiteId> {
    pub prog_id: Id,
    pub offset: u16,
}

impl<Id: SiteId> Site<Id> {
    #[inline]
    pub fn new(prog_id: Id, offset: u16) -> Self { Self { prog_id, offset } }
}

impl<Id: SiteId> Display for Site<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{}:{:04X}.h", self.prog_id, self.offset) }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[repr(u8)]
pub(super) enum Idx16 {
    #[display(":1")]
    L1 = 0,
    #[display(":2")]
    L2 = 1,
    #[display(":3")]
    L3 = 2,
    #[display(":4")]
    L4 = 3,
    #[display(":5")]
    L5 = 4,
    #[display(":6")]
    L6 = 5,
    #[display(":7")]
    L7 = 6,
    #[display(":8")]
    L8 = 7,
    #[display(":9")]
    L9 = 8,
    #[display(":10")]
    L10 = 9,

    #[display(":A")]
    A = 0xA,
    #[display(":B")]
    B = 0xB,
    #[display(":C")]
    C = 0xC,
    #[display(":D")]
    D = 0xD,
    #[display(":E")]
    E = 0xE,
    #[display(":F")]
    F = 0xF,
}

impl Idx16 {
    pub const ALL: [Self; 16] = [
        Self::L1,
        Self::L2,
        Self::L3,
        Self::L4,
        Self::L5,
        Self::L6,
        Self::L7,
        Self::L8,
        Self::L9,
        Self::L10,
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
    ];

    pub(super) fn from_expected(val: usize) -> Self {
        for i in Self::ALL {
            if i as usize == val {
                return i;
            }
        }
        panic!("invalid 4-bit integer index represented in a usize value")
    }
}

impl From<u4> for Idx16 {
    fn from(idx: u4) -> Self {
        for i in Self::ALL {
            if i as u8 == idx.to_u8() {
                return i;
            }
        }
        unreachable!()
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[repr(u8)]
pub(super) enum Idx32 {
    #[display(":1")]
    L1 = 0,
    #[display(":2")]
    L2 = 1,
    #[display(":3")]
    L3 = 2,
    #[display(":4")]
    L4 = 3,
    #[display(":5")]
    L5 = 4,
    #[display(":6")]
    L6 = 5,
    #[display(":7")]
    L7 = 6,
    #[display(":8")]
    L8 = 7,
    #[display(":9")]
    L9 = 8,
    #[display(":10")]
    L10 = 9,

    #[display(":A")]
    A = 0xA,
    #[display(":B")]
    B = 0xB,
    #[display(":C")]
    C = 0xC,
    #[display(":D")]
    D = 0xD,
    #[display(":E")]
    E = 0xE,
    #[display(":F")]
    F = 0xF,

    #[display(".g")]
    Sg = 0x10,
    #[display(".h")]
    Sh = 0x11,
    #[display(".k")]
    Sk = 0x12,
    #[display(".m")]
    Sm = 0x13,
    #[display(".n")]
    Sn = 0x14,
    #[display(".p")]
    Sp = 0x15,
    #[display(".q")]
    Sq = 0x16,
    #[display(".r")]
    Sr = 0x17,
    #[display(".s")]
    Ss = 0x18,
    #[display(".t")]
    St = 0x19,
    #[display(".u")]
    Su = 0x1A,
    #[display(".v")]
    Sv = 0x1B,
    #[display(".w")]
    Sw = 0x1C,
    #[display(".x")]
    Sx = 0x1D,
    #[display(".y")]
    Sy = 0x1E,
    #[display(".z")]
    Sz = 0x1F,
}

impl Idx32 {
    pub const ALL: [Self; 32] = [
        Self::L1,
        Self::L2,
        Self::L3,
        Self::L4,
        Self::L5,
        Self::L6,
        Self::L7,
        Self::L8,
        Self::L9,
        Self::L10,
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::Sg,
        Self::Sh,
        Self::Sk,
        Self::Sm,
        Self::Sn,
        Self::Sp,
        Self::Sq,
        Self::Sr,
        Self::Ss,
        Self::St,
        Self::Su,
        Self::Sv,
        Self::Sw,
        Self::Sx,
        Self::Sy,
        Self::Sz,
    ];

    pub(super) fn from_expected(val: usize) -> Self {
        for i in Self::ALL {
            if i as usize == val {
                return i;
            }
        }
        panic!("invalid 5-bit integer index represented in a usize value")
    }
}

impl From<u5> for Idx32 {
    fn from(idx: u5) -> Self {
        for i in Self::ALL {
            if i as u8 == idx.to_u8() {
                return i;
            }
        }
        unreachable!()
    }
}
