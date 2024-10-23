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
use crate::core::{RegA, SiteId, A};
use crate::isa::bytecode::CodeEofError;
#[cfg(feature = "GFA")]
use crate::isa::FieldInstr;
use crate::isa::{Bytecode, BytecodeRead, BytecodeWrite, Instr, InstructionSet, ReservedInstr};
use crate::Site;

impl<Id: SiteId, Ext: InstructionSet<Id> + Bytecode<Id>> Bytecode<Id> for Instr<Id, Ext> {
    fn op_range() -> RangeInclusive<u8> { 0..=0xFF }

    fn opcode_byte(&self) -> u8 {
        match self {
            Instr::Ctrl(instr) => instr.opcode_byte(),
            Instr::Reg(instr) => Bytecode::<Id>::opcode_byte(instr),
            #[cfg(feature = "GFA")]
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
            #[cfg(feature = "GFA")]
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
        match opcode {
            op if CtrlInstr::<Id>::op_range().contains(&op) => {
                CtrlInstr::<Id>::decode_operands(reader, op).map(Self::Ctrl)
            }
            op if <RegInstr as Bytecode<Id>>::op_range().contains(&op) => {
                <RegInstr as Bytecode<Id>>::decode_operands(reader, op).map(Self::Reg)
            }
            #[cfg(feature = "GFA")]
            op if <FieldInstr as Bytecode<Id>>::op_range().contains(&op) => {
                <FieldInstr as Bytecode<Id>>::decode_operands(reader, op).map(Self::GFqA)
            }
            0x80..=0xFF => Ext::decode_operands(reader, opcode).map(Self::Ext),
            _ => ReservedInstr::decode_operands(reader, opcode).map(Self::Reserved),
        }
    }
}

impl<Id: SiteId> Bytecode<Id> for ReservedInstr {
    fn op_range() -> RangeInclusive<u8> { 0..=0x7F }

    fn opcode_byte(&self) -> u8 { self.0 }

    fn encode_operands<W>(&self, _writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        Ok(())
    }

    fn decode_operands<R>(_reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        Ok(ReservedInstr(opcode))
    }
}

impl<Id: SiteId> CtrlInstr<Id> {
    const START: u8 = 0;
    const END: u8 = Self::START + Self::STOP;

    const NOP: u8 = 0;
    const NOCO: u8 = 1;
    const CHK: u8 = 2;
    const FAIL: u8 = 3;
    const RSET: u8 = 4;
    const JMP: u8 = 5;
    const JIFNE: u8 = 6;
    const JIFAIL: u8 = 7;
    const SH: u8 = 8;
    const SHNE: u8 = 9;
    const SHFAIL: u8 = 10;
    const EXEC: u8 = 11;
    const FN: u8 = 12;
    const CALL: u8 = 13;
    const RET: u8 = 14;
    const STOP: u8 = 15;
}

impl<Id: SiteId> Bytecode<Id> for CtrlInstr<Id> {
    fn op_range() -> RangeInclusive<u8> { Self::START..=Self::END }

    fn opcode_byte(&self) -> u8 {
        match self {
            CtrlInstr::Nop => Self::NOP,
            CtrlInstr::Chk => Self::CHK,
            CtrlInstr::NotCo => Self::NOCO,
            CtrlInstr::FailCk => Self::FAIL,
            CtrlInstr::RsetCk => Self::RSET,
            CtrlInstr::Jmp { .. } => Self::JMP,
            CtrlInstr::JifCo { .. } => Self::JIFNE,
            CtrlInstr::JifCk { .. } => Self::JIFAIL,
            CtrlInstr::Sh { .. } => Self::SH,
            CtrlInstr::ShNe { .. } => Self::SHNE,
            CtrlInstr::ShFail { .. } => Self::SHFAIL,
            CtrlInstr::Exec { .. } => Self::EXEC,
            CtrlInstr::Fn { .. } => Self::FN,
            CtrlInstr::Call { .. } => Self::CALL,
            CtrlInstr::Ret => Self::RET,
            CtrlInstr::Stop => Self::STOP,
        }
    }

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
            CtrlInstr::Sh { shift } | CtrlInstr::ShNe { shift } | CtrlInstr::ShFail { shift } => {
                writer.write_fixed(shift.to_le_bytes())?
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
        Ok(match opcode {
            Self::NOP => Self::Nop,
            Self::CHK => Self::Chk,
            Self::FAIL => Self::FailCk,
            Self::RSET => Self::RsetCk,
            Self::NOCO => Self::NotCo,
            Self::RET => Self::Ret,
            Self::STOP => Self::Stop,

            Self::JMP => CtrlInstr::Jmp {
                pos: reader.read_fixed(u16::from_le_bytes)?,
            },
            Self::JIFNE => CtrlInstr::JifCo {
                pos: reader.read_fixed(u16::from_le_bytes)?,
            },
            Self::JIFAIL => CtrlInstr::JifCk {
                pos: reader.read_fixed(u16::from_le_bytes)?,
            },
            Self::FN => CtrlInstr::Fn {
                pos: reader.read_fixed(u16::from_le_bytes)?,
            },

            Self::SH => CtrlInstr::Sh {
                shift: reader.read_fixed(i8::from_le_bytes)?,
            },
            Self::SHNE => CtrlInstr::ShNe {
                shift: reader.read_fixed(i8::from_le_bytes)?,
            },
            Self::SHFAIL => CtrlInstr::ShFail {
                shift: reader.read_fixed(i8::from_le_bytes)?,
            },

            Self::CALL => {
                let prog_id = reader.read_ref()?;
                let offset = reader.read_word()?;
                let site = Site::new(prog_id, offset);
                CtrlInstr::Call { site }
            }
            Self::EXEC => {
                let prog_id = reader.read_ref()?;
                let offset = reader.read_word()?;
                let site = Site::new(prog_id, offset);
                CtrlInstr::Exec { site }
            }

            _ => unreachable!(),
        })
    }
}

impl RegInstr {
    const START: u8 = 16;
    const END: u8 = Self::START + Self::EQ;

    const CLR: u8 = 16;
    const PUT: u8 = 17;
    const PIF: u8 = 18;
    const TEST: u8 = 19;
    const CPY: u8 = 20;
    const SWP: u8 = 21;
    const EQ: u8 = 22;
}

impl<Id: SiteId> Bytecode<Id> for RegInstr {
    fn op_range() -> RangeInclusive<u8> { Self::START..=Self::END }

    fn opcode_byte(&self) -> u8 {
        match self {
            RegInstr::Clr { .. } => Self::CLR,
            RegInstr::Put { .. } => Self::PUT,
            RegInstr::Pif { .. } => Self::PIF,
            RegInstr::Test { .. } => Self::TEST,
            RegInstr::Cpy { .. } => Self::CPY,
            RegInstr::Swp { .. } => Self::SWP,
            RegInstr::Eq { .. } => Self::EQ,
        }
    }

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
        Ok(match opcode {
            RegInstr::CLR => {
                let dst = RegA::from(reader.read_byte()?);
                RegInstr::Clr { dst }
            }
            RegInstr::PUT | RegInstr::PIF => {
                let dst = RegA::from(reader.read_byte()?);
                let val = match dst.a() {
                    A::A8 => reader.read_byte().map(|v| v as u128),
                    A::A16 => reader.read_word().map(|v| v as u128),
                    A::A32 => reader.read_fixed(u32::from_le_bytes).map(|v| v as u128),
                    A::A64 => reader.read_fixed(u64::from_le_bytes).map(|v| v as u128),
                    A::A128 => reader.read_fixed(u128::from_le_bytes),
                }
                .ok()
                .into();

                if opcode == RegInstr::PUT {
                    RegInstr::Put { dst, val }
                } else {
                    RegInstr::Pif { dst, val }
                }
            }
            RegInstr::TEST => {
                let src = RegA::from(reader.read_byte()?);
                RegInstr::Test { src }
            }
            RegInstr::CPY => {
                let dst = RegA::from(reader.read_byte()?);
                let src = RegA::from(reader.read_byte()?);
                RegInstr::Cpy { dst, src }
            }
            RegInstr::SWP => {
                let src_dst1 = RegA::from(reader.read_byte()?);
                let src_dst2 = RegA::from(reader.read_byte()?);
                RegInstr::Swp { src_dst1, src_dst2 }
            }
            RegInstr::EQ => {
                let src1 = RegA::from(reader.read_byte()?);
                let src2 = RegA::from(reader.read_byte()?);
                RegInstr::Eq { src1, src2 }
            }
            _ => unreachable!(),
        })
    }
}
