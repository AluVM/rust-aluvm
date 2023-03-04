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

//! Constants defined for AluVM libraries

#![allow(missing_docs)]

pub const CODE_SEGMENT_MAX_LEN: usize = 1 << 16;

pub const DATA_SEGMENT_MAX_LEN: usize = 1 << 16;

/// Maximum number of libraries that may be referenced (used by) any other library; i.e. limit for
/// the number of records inside program segment.
pub const LIBS_SEGMENT_MAX_COUNT: usize = 1 << 8;

/// Maximum total number of libraries which may be used by a single program; i.e. maximal number of
/// nodes in a library dependency tree.
pub const LIBS_MAX_TOTAL: u16 = 1024;

pub const ISAE_SEGMENT_MAX_LEN: usize = 0xFF;

pub const ISAE_SEGMENT_MAX_COUNT: usize = 32;

pub const ISA_ID_MIN_LEN: usize = 2;

pub const ISA_ID_MAX_LEN: usize = 8;

pub const ISA_ID_ALLOWED_CHARS: [char; 36] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
pub const ISA_ID_ALLOWED_FIRST_CHAR: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub const ISA_ID_ALU: &str = "ALU";
pub const ISA_ID_BPDIGEST: &str = "BPDIGEST";
pub const ISA_ID_SECP256K: &str = "SECP256K";
pub const ISA_ID_ED25519: &str = "ED25519";

pub const ISA_ID_ALURE: &str = "ALURE";
pub const ISA_ID_SIMD: &str = "SIMD";
pub const ISA_ID_INET2: &str = "INET4";
pub const ISA_ID_WEB4: &str = "WEB4";

pub const ISA_ID_BITCOIN: &str = "BITCOIN";
pub const ISA_ID_BP: &str = "BP";
pub const ISA_ID_RGB: &str = "RGB";
pub const ISA_ID_LNP: &str = "LNP";

pub const ISA_ID_REBICA: &str = "REBICA";
