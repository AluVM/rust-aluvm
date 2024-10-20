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

use core::ops::RangeInclusive;

use super::{ArithmInstr, CtrlInstr, RegInstr};
use crate::isa::bytecode::CodeEofError;
use crate::isa::{Bytecode, BytecodeRead, BytecodeWrite, Instr, InstructionSet, ReservedInstr};

impl<Id, Ext: InstructionSet> Bytecode<Id> for Instr<Ext> {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        todo!()
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}

impl<Id> Bytecode<Id> for ReservedInstr {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        todo!()
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}

impl<Id> Bytecode<Id> for CtrlInstr {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        todo!()
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}

impl<Id> Bytecode<Id> for RegInstr {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        todo!()
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}

impl<Id> Bytecode<Id> for ArithmInstr {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        todo!()
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}
