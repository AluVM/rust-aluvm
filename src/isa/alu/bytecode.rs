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

use super::{CtrlInstr, MaybeU128, RegInstr};
use crate::core::{SiteId, A};
use crate::isa::bytecode::CodeEofError;
use crate::isa::{Bytecode, BytecodeRead, BytecodeWrite, Instr, InstructionSet, ReservedInstr};
use crate::Site;

impl<Id: SiteId, Ext: InstructionSet<Id> + Bytecode<Id>> Bytecode<Id> for Instr<Id, Ext> {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 {
        match self {
            Instr::Ctrl(instr) => instr.opcode_byte(),
            Instr::Reg(instr) => Bytecode::<Id>::opcode_byte(instr),
            Instr::GFqA(instr) => Bytecode::<Id>::opcode_byte(instr),
            Instr::Reserved(instr) => Bytecode::<Id>::opcode_byte(instr),
            Instr::Ext(instr) => instr.opcode_byte(),
        }
    }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        match self {
            Instr::Ctrl(instr) => instr.encode_operands(writer),
            Instr::Reg(instr) => instr.encode_operands(writer),
            Instr::GFqA(instr) => instr.encode_operands(writer),
            Instr::Reserved(instr) => instr.encode_operands(writer),
            Instr::Ext(instr) => instr.encode_operands(writer),
        }
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}

impl<Id: SiteId> Bytecode<Id> for ReservedInstr {
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

impl<Id: SiteId> Bytecode<Id> for CtrlInstr<Id> {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        match *self {
            CtrlInstr::Nop
            | CtrlInstr::Chk
            | CtrlInstr::FailCk
            | CtrlInstr::RsetCk
            | CtrlInstr::NotCo
            | CtrlInstr::Ret
            | CtrlInstr::Stop => {}

            CtrlInstr::Jmp { pos } | CtrlInstr::JifCo { pos } | CtrlInstr::JifCk { pos } | CtrlInstr::Fn { pos } => {
                writer.write_fixed(pos.to_le_bytes())?
            }
            CtrlInstr::Shift { shift } | CtrlInstr::ShIfCo { shift } | CtrlInstr::ShIfCk { shift } => {
                writer.write_byte(shift.to_le_bytes()[0])?
            }
            CtrlInstr::Call { site } | CtrlInstr::Exec { site } => {
                let site = Site::new(site.prog_id, site.offset);
                writer.write_ref(site.prog_id)?;
                writer.write_word(site.offset)?;
            }
        }
        Ok(())
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}

impl<Id: SiteId> Bytecode<Id> for RegInstr {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        match *self {
            RegInstr::Clr { dst } => {
                writer.write_byte(dst.to_u8())?;
            }
            RegInstr::Put { dst, val } | RegInstr::Pif { dst, val } => {
                writer.write_byte(dst.to_u8())?;
                let MaybeU128::U128(val) = val else {
                    panic!("an attempt to serialize program with missed data");
                };
                match dst.a() {
                    A::A8 => writer.write_byte(val as u8)?,
                    A::A16 => writer.write_word(val as u16)?,
                    A::A32 => writer.write_fixed(val.to_le_bytes())?,
                    A::A64 => writer.write_fixed(val.to_le_bytes())?,
                    A::A128 => writer.write_fixed(val.to_le_bytes())?,
                }
            }
            RegInstr::Test { src } => {
                writer.write_byte(src.to_u8())?;
            }
            RegInstr::Cpy { dst, src } => {
                writer.write_byte(dst.to_u8())?;
                writer.write_byte(src.to_u8())?;
            }
            RegInstr::Swp { src_dst1, src_dst2 } => {
                writer.write_byte(src_dst1.to_u8())?;
                writer.write_byte(src_dst2.to_u8())?;
            }
            RegInstr::Eq { src1, src2 } => {
                writer.write_byte(src1.to_u8())?;
                writer.write_byte(src2.to_u8())?;
            }
        }
        Ok(())
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        todo!()
    }
}
