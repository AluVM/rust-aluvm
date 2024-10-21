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

use std::ops::RangeInclusive;

use amplify::num::u1;

use super::FieldInstr;
use crate::core::SiteId;
use crate::isa::{Bytecode, BytecodeRead, BytecodeWrite, CodeEofError};

impl<Id: SiteId> Bytecode<Id> for FieldInstr {
    fn op_range() -> RangeInclusive<u8> { todo!() }

    fn opcode_byte(&self) -> u8 { todo!() }

    fn encode_operands<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where W: BytecodeWrite<Id> {
        match *self {
            FieldInstr::IncMod { src_dst, val } => {
                writer.write_u8(src_dst.to_u8())?;
                writer.write_u8(val)?;
            }
            FieldInstr::DecMod { src_dst, val } => {
                writer.write_u8(src_dst.to_u8())?;
                writer.write_u8(val)?;
            }
            FieldInstr::NegMod { src_dst } => {
                writer.write_u8(src_dst.to_u8())?;
            }
            FieldInstr::AddMod { reg, dst, src1, src2 } => {
                writer.write_u1(u1::ZERO)?;
                writer.write_u3(reg.to_u3())?;
                writer.write_u4(dst.to_u4())?;
                writer.write_u4(src1.to_u4())?;
                writer.write_u4(src2.to_u4())?;
            }
            FieldInstr::MulMod { reg, dst, src1, src2 } => {
                writer.write_u1(u1::ONE)?;
                writer.write_u3(reg.to_u3())?;
                writer.write_u4(dst.to_u4())?;
                writer.write_u4(src1.to_u4())?;
                writer.write_u4(src2.to_u4())?;
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
