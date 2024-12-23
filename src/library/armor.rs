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

use ::armor::{ArmorHeader, ArmorParseError, AsciiArmor, ASCII_ARMOR_ID};
use amplify::confinement::{self, Confined, U24 as U24MAX};
use strict_encoding::{DeserializeError, StrictDeserialize, StrictSerialize};

use super::*;

const ASCII_ARMOR_ISAE: &str = "ISA-Extensions";
const ASCII_ARMOR_DEPENDENCY: &str = "Dependency";

/// Errors while deserializing lib-old from an ASCII Armor.
#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(inner)]
pub enum LibArmorError {
    /// Armor parse error.
    #[from]
    Armor(ArmorParseError),

    /// The provided data exceed maximum possible lib-old size.
    #[from(confinement::Error)]
    TooLarge,

    /// Library data deserialization error.
    #[from]
    Decode(DeserializeError),
}

impl AsciiArmor for Lib {
    type Err = LibArmorError;
    const PLATE_TITLE: &'static str = "ALUVM LIB";

    fn ascii_armored_headers(&self) -> Vec<ArmorHeader> {
        let mut headers = vec![
            ArmorHeader::new(ASCII_ARMOR_ID, self.lib_id().to_string()),
            ArmorHeader::new(ASCII_ARMOR_ISAE, self.isae_string()),
        ];
        for dep in &self.libs {
            headers.push(ArmorHeader::new(ASCII_ARMOR_DEPENDENCY, dep.to_string()));
        }
        headers
    }

    fn to_ascii_armored_data(&self) -> Vec<u8> {
        self.to_strict_serialized::<U24MAX>()
            .expect("type guarantees")
            .to_vec()
    }

    fn with_headers_data(_headers: Vec<ArmorHeader>, data: Vec<u8>) -> Result<Self, Self::Err> {
        // TODO: check id, dependencies and ISAE
        let data = Confined::try_from(data)?;
        let me = Self::from_strict_serialized::<U24MAX>(data)?;
        Ok(me)
    }
}
