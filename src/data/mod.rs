// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Institute. All rights reserved.
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

//! Internal data representations and operations on data used by AluVM

mod arithm;
mod bitwise;
mod byte_str;
#[cfg(feature = "std")]
pub mod encoding;
mod number;

pub use byte_str::ByteStr;
pub use number::{
    FloatLayout, IntLayout, Layout, LiteralParseError, MaybeNumber, Number, NumberLayout, Step,
};

/// Value which can be extracted from any register.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
pub enum RegValue {
    /// Value extracted from numerical registers
    #[from]
    #[from(Number)]
    Number(MaybeNumber),

    /// Value extracted from string register
    #[from]
    #[from(ByteStr)]
    String(Option<ByteStr>),
}

mod display {
    use core::fmt::{self, Display, Formatter};

    use super::*;

    impl Display for RegValue {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                RegValue::Number(n) => Display::fmt(n, f),
                RegValue::String(Some(s)) => Display::fmt(s, f),
                RegValue::String(None) => f.write_str("~"),
            }
        }
    }
}
