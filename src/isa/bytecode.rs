// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Labs,
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

use std::ops::RangeInclusive;

use crate::isa::alu::{ArithmInstr, BitInstr, CtrlInstr, RegInstr, SignedInstr};
use crate::isa::{Instr, ReservedInstr};
use crate::library::{Bytecode, BytecodeError, CodeEofError, InstructionSet, Read, Write};

impl<Extension: InstructionSet> Bytecode for Instr<Extension> {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}

impl Bytecode for ReservedInstr {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}

impl Bytecode for CtrlInstr {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}

impl Bytecode for RegInstr {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}

impl Bytecode for ArithmInstr {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}

impl Bytecode for SignedInstr {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}

impl Bytecode for BitInstr {
    fn instr_range() -> RangeInclusive<u8> { todo!() }

    fn instr_byte(&self) -> u8 { todo!() }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        todo!()
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: Read,
    {
        todo!()
    }
}
