// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2021-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 LNP/BP Standards Association, Switzerland.
// Copyright (C) 2024-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2021-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

//! Strict types lib-old generator methods.

use core::convert::TryFrom;

use strict_types::typelib::{CompileError, LibBuilder};
use strict_types::TypeLib;

use crate::{CoreConfig, Lib, LibSite, LIB_NAME_ALUVM};

/// Strict type id for the lib-old providing data types from this crate.
pub const LIB_ID_ALUVM: &str =
    "stl:zkW0z6BJ-O8DoUgR-47B0fPf-diV1Rzs-XpiHP5n-FE~MRiE#yogurt-cricket-frog";

fn _aluvm_stl() -> Result<TypeLib, CompileError> {
    LibBuilder::new(libname!(LIB_NAME_ALUVM), tiny_bset! {
        strict_types::stl::std_stl().to_dependency(),
        strict_types::stl::strict_types_stl().to_dependency()
    })
    .transpile::<LibSite>()
    .transpile::<Lib>()
    .transpile::<CoreConfig>()
    .compile()
}

/// Generates strict type lib-old providing data types from this crate.
pub fn aluvm_stl() -> TypeLib { _aluvm_stl().expect("invalid strict type AluVM lib-old") }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lib_id() {
        let lib = aluvm_stl();
        assert_eq!(lib.id().to_string(), LIB_ID_ALUVM);
    }
}
