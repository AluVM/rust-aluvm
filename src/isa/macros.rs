// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2024 UBIDECO Labs,
//     Institute for Distributed and Cognitive Computing, Switzerland.
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

#[allow(unused_macros)]
macro_rules! A {
    [$reg:ident @ $core:ident] => {
        checked!($core.a($reg))
    };
    [$a:ident : $idx:ident @ $core:ident] => {{
        checked!($core.a(RegA::with($a, $idx.into())))
    }};
}

#[allow(unused_macros)]
macro_rules! checked {
    ($core:ident . $op:ident($($arg:expr),*)) => {{
        let Some(val) = $core.$op( $( $arg ),* ) else {
            return $crate::ExecStep::FailContinue;
        };
        val
    }};
}
