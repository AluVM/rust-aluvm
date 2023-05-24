// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2023 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023 UBIDECO Institute. All rights reserved.
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

//! Strict types library generator methods.

use strict_types::typelib::{LibBuilder, TranslateError};
use strict_types::TypeLib;

use crate::library::LibSite;
use crate::LIB_NAME_ALUVM;

/// Strict type id for the library providing data types from this crate.
pub const LIB_ID_ALUVM: &str = "edition_cuba_sahara_9njuRyHw1Jt7XpohKc7LuNR92R9DWQGvuygn5XJgzpzP";

fn _aluvm_stl() -> Result<TypeLib, TranslateError> {
    LibBuilder::new(libname!(LIB_NAME_ALUVM), tiny_bset! {
        strict_types::stl::strict_types_stl().to_dependency()
    })
    .transpile::<LibSite>()
    .compile()
}

/// Generates strict type library providing data types from this crate.
pub fn aluvm_stl() -> TypeLib { _aluvm_stl().expect("invalid strict type AluVM library") }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lib_id() {
        let lib = aluvm_stl();
        assert_eq!(lib.id().to_string(), LIB_ID_ALUVM);
    }
}
