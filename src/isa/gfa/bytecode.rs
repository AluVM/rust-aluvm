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

use amplify::num::u1;

use super::FieldInstr;
use crate::core::{IdxAl, RegA, SiteId, A};
use crate::isa::{Bytecode, BytecodeRead, BytecodeWrite, CodeEofError};

impl FieldInstr {
    const START: u8 = 64;
    const END: u8 = Self::START + Self::ADD_MUL;
    const INC_MOD: u8 = 0;
    const DEC_MOD: u8 = 1;
    const NEG_MOD: u8 = 2;
    const ADD_MUL: u8 = 3;
}

impl<Id: SiteId> Bytecode<Id> for FieldInstr {
    fn op_range() -> RangeInclusive<u8> { Self::START..=Self::END }

    fn opcode_byte(&self) -> u8 {
        Self::START
            + match *self {
                FieldInstr::IncMod { .. } => Self::INC_MOD,
                FieldInstr::DecMod { .. } => Self::DEC_MOD,
                FieldInstr::NegMod { .. } => Self::NEG_MOD,
                FieldInstr::AddMod { .. } | FieldInstr::MulMod { .. } => Self::ADD_MUL,
            }
    }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        match *self {
            FieldInstr::IncMod { src_dst, val } => {
                writer.write_byte(src_dst.to_u8())?;
                writer.write_byte(val)?;
            }
            FieldInstr::DecMod { src_dst, val } => {
                writer.write_byte(src_dst.to_u8())?;
                writer.write_byte(val)?;
            }
            FieldInstr::NegMod { src_dst } => {
                writer.write_byte(src_dst.to_u8())?;
            }
            FieldInstr::AddMod { reg, dst, src1, src2 } => {
                writer.write_1bit(u1::ZERO)?;
                writer.write_3bits(reg.to_u3())?;
                writer.write_4bits(dst.to_u4())?;
                writer.write_4bits(src1.to_u4())?;
                writer.write_4bits(src2.to_u4())?;
            }
            FieldInstr::MulMod { reg, dst, src1, src2 } => {
                writer.write_1bit(u1::ONE)?;
                writer.write_3bits(reg.to_u3())?;
                writer.write_4bits(dst.to_u4())?;
                writer.write_4bits(src1.to_u4())?;
                writer.write_4bits(src2.to_u4())?;
            }
        }
        Ok(())
    }

    fn decode_operands<R>(reader: &mut R, opcode: u8) -> Result<Self, CodeEofError>
    where
        Self: Sized,
        R: BytecodeRead<Id>,
    {
        Ok(match opcode - Self::START {
            Self::INC_MOD => {
                let src_dst = RegA::from(reader.read_byte()?);
                let val = reader.read_byte()?;
                FieldInstr::IncMod { src_dst, val }
            }
            Self::DEC_MOD => {
                let src_dst = RegA::from(reader.read_byte()?);
                let val = reader.read_byte()?;
                FieldInstr::IncMod { src_dst, val }
            }
            Self::NEG_MOD => {
                let src_dst = RegA::from(reader.read_byte()?);
                FieldInstr::NegMod { src_dst }
            }
            Self::ADD_MUL => {
                let subop = reader.read_1bit()?;
                let reg = A::from(reader.read_3bits()?);
                let dst = IdxAl::from(reader.read_4bits()?);
                let src1 = IdxAl::from(reader.read_4bits()?);
                let src2 = IdxAl::from(reader.read_4bits()?);
                match subop {
                    u1::ZERO => FieldInstr::AddMod { reg, dst, src1, src2 },
                    u1::ONE => FieldInstr::MulMod { reg, dst, src1, src2 },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        })
    }
}
