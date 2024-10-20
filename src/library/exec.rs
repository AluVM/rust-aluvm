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

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::{String, ToString};

use baid64::DisplayBaid64;

use super::{Bytecode, Cursor, IsaName, IsaSeg, Lib, LibSite, Read};
use crate::reg::{CoreRegs, Reg, Register};

/// Turing machine movement after instruction execution
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ExecStep {
    /// Stop program execution
    Stop,

    /// Stop and fail program execution
    Fail,

    /// Move to the next instruction
    Next,

    /// Jump to the offset from the origin
    Jump(u16),

    /// Jump to another code fragment
    Call(LibSite),
}

/// Trait for instructions
pub trait InstructionSet: Bytecode + core::fmt::Display + core::fmt::Debug {
    /// Context: external data which are accessible to the ISA.
    type Context<'ctx>;

    /// ISA Extensions used by the provided instruction set.
    ///
    /// Each id must be up to 8 bytes and consist of upper case latin alphanumeric characters,
    /// starting with non-number.
    fn isa_ids() -> IsaSeg;

    /// ISA Extension IDs represented as a standard string (space-separated)
    ///
    /// Concatenated length of the ISA IDs joined via ' ' character must not exceed 128 bytes.
    #[inline]
    fn isa_string() -> String { Self::isa_ids().to_string() }

    /// ISA Extension IDs encoded in a standard way (space-separated)
    ///
    /// Concatenated length of the ISA IDs joined via ' ' character must not exceed 128 bytes.
    #[inline]
    fn isa_id() -> Box<[u8]> { Self::isa_string().as_bytes().into() }

    /// Checks whether provided ISA extension ID is supported by the current instruction set
    #[inline]
    fn is_supported(id: &IsaName) -> bool { Self::isa_ids().contains(id) }

    /// Lists all registers which are used by the instruction.
    fn regs(&self) -> BTreeSet<Reg> {
        let mut regs = self.src_regs();
        regs.extend(self.dst_regs());
        regs
    }

    /// List of registers which value is taken into the account by the instruction.
    fn src_regs(&self) -> BTreeSet<Reg>;

    /// List of registers which value may be changed by the instruction.
    fn dst_regs(&self) -> BTreeSet<Reg>;

    /// Returns computational complexity of the instruction.
    ///
    /// Computational complexity is the number of "CPU ticks" required to process the instruction.
    fn complexity(&self) -> u64 {
        // By default, give the upper estimate
        self.src_regs().iter().chain(&self.dst_regs()).map(|reg| reg.bytes() as u64).sum::<u64>()
            * 100
    }

    /// Executes given instruction taking all registers as input and output.
    ///
    /// # Arguments
    ///
    /// The method is provided with the current code position which may be used by the instruction
    /// for constructing call stack.
    ///
    /// # Returns
    ///
    /// Returns whether further execution should be stopped.
    // TODO: Take the instruction by reference
    fn exec(&self, regs: &mut CoreRegs, site: LibSite, context: &Self::Context<'_>) -> ExecStep;
}

impl Lib {
    /// Executes library code starting at entrypoint
    ///
    /// # Returns
    ///
    /// Location for the external code jump, if any
    pub fn exec<Isa>(
        &self,
        entrypoint: u16,
        registers: &mut CoreRegs,
        context: &Isa::Context<'_>,
    ) -> Option<LibSite>
    where
        Isa: InstructionSet,
    {
        #[cfg(feature = "log")]
        let (m, w, d, g, r, y, z) = (
            "\x1B[0;35m",
            "\x1B[1;1m",
            "\x1B[0;37;2m",
            "\x1B[0;32m",
            "\x1B[0;31m",
            "\x1B[0;33m",
            "\x1B[0m",
        );

        let mut cursor = Cursor::with(&self.code, &self.data, &self.libs);
        let lib_id = self.id();

        #[cfg(feature = "log")]
        let lib_mnemonic = lib_id.to_baid64_mnemonic();
        #[cfg(feature = "log")]
        let lib_ref = lib_mnemonic.split_at(5).0;

        if cursor.seek(entrypoint).is_err() {
            registers.st0 = false;
            #[cfg(feature = "log")]
            eprintln!("jump to non-existing offset; halting, {d}st0{z} is set to {r}false{z}");
            return None;
        }

        #[cfg(feature = "log")]
        let mut st0 = registers.st0;

        while !cursor.is_eof() {
            let pos = cursor.pos();

            let instr = Isa::decode(&mut cursor).ok()?;

            #[cfg(feature = "log")]
            {
                eprint!("{m}{}@x{pos:06X}:{z} {: <32}; ", lib_ref, instr.to_string());
                for reg in instr.src_regs() {
                    let val = registers.get(reg);
                    eprint!("{d}{reg}={z}{w}{val}{z} ");
                }
            }

            let next = instr.exec(registers, LibSite::with(pos, lib_id), context);

            #[cfg(feature = "log")]
            {
                eprint!("-> ");
                for reg in instr.dst_regs() {
                    let val = registers.get(reg);
                    eprint!("{g}{reg}={y}{val}{z} ");
                }
                if st0 != registers.st0 {
                    let c = if registers.st0 { g } else { r };
                    eprint!(" {d}st0={z}{c}{}{z} ", registers.st0);
                }

                st0 = registers.st0;
            }

            if !registers.acc_complexity(instr) {
                #[cfg(feature = "log")]
                eprintln!("complexity overflow");
                return None;
            }
            match next {
                ExecStep::Stop => {
                    #[cfg(feature = "log")]
                    {
                        let c = if registers.st0 { g } else { r };
                        eprintln!("execution stopped; {d}st0={z}{c}{}{z}", registers.st0);
                    }
                    return None;
                }
                ExecStep::Fail => {
                    registers.st0 = false;
                    assert_eq!(registers.st0, false);
                    #[cfg(feature = "log")]
                    eprintln!("halting, {d}st0{z} is set to {r}false{z}");
                    return None;
                }
                ExecStep::Next => {
                    #[cfg(feature = "log")]
                    eprintln!();
                    continue;
                }
                ExecStep::Jump(pos) => {
                    #[cfg(feature = "log")]
                    eprintln!("{}", pos);
                    if cursor.seek(pos).is_err() {
                        registers.st0 = false;
                        #[cfg(feature = "log")]
                        eprintln!(
                            "jump to non-existing offset; halting, {d}st0{z} is set to {r}false{z}"
                        );
                        return None;
                    }
                }
                ExecStep::Call(site) => {
                    #[cfg(feature = "log")]
                    eprintln!("{}", site);
                    return Some(site);
                }
            }
        }

        None
    }
}
