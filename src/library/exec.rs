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

#[cfg(feature = "log")]
use baid64::DisplayBaid64;

use super::{Lib, Marshaller};
use crate::isa::{Bytecode, BytecodeRead, ExecStep, Instruction};
use crate::{Core, LibId, LibSite, Site};

impl Lib {
    /// Execute library code starting at entrypoint.
    ///
    /// # Returns
    ///
    /// Location for the external code jump, if any.
    pub fn exec<Instr>(
        &self,
        entrypoint: u16,
        registers: &mut Core<LibId, Instr::Core>,
        context: &Instr::Context<'_>,
    ) -> Option<LibSite>
    where
        Instr: Instruction<LibId> + Bytecode<LibId>,
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

        let mut marshaller = Marshaller::with(&self.code, &self.data, &self.libs);
        let lib_id = self.lib_id();

        #[cfg(feature = "log")]
        let lib_mnemonic = lib_id.to_baid64_mnemonic();
        #[cfg(feature = "log")]
        let lib_ref = lib_mnemonic.split_at(5).0;

        if marshaller.seek(entrypoint).is_err() {
            registers.reset_ck();
            #[cfg(feature = "log")]
            eprintln!("jump to non-existing offset; halting, {d}st0{z} is set to {r}false{z}");
            return None;
        }

        #[cfg(feature = "log")]
        let mut ck0 = registers.ck();

        while !marshaller.is_eof() {
            let pos = marshaller.pos();

            let instr = Instr::decode_instr(&mut marshaller).ok()?;

            #[cfg(feature = "log")]
            {
                eprint!("{m}{}@x{pos:06X}:{z} {: <32}; ", lib_ref, instr.to_string());
                for reg in instr.src_regs() {
                    let val = registers.get(reg);
                    eprint!("{d}{reg} {z}{w}{}{z}, ", val);
                }
            }

            let next = instr.exec(registers, Site::new(lib_id, pos), context);

            #[cfg(feature = "log")]
            {
                eprint!("-> ");
                for reg in instr.dst_regs() {
                    let val = registers.get(reg);
                    eprint!("{g}{reg} {y}{}{z}, ", val);
                }
                if ck0 != registers.ck() {
                    let c = if registers.ck().is_ok() { g } else { r };
                    eprint!(" {d}CK {z}{c}{}{z}, ", registers.ck());
                }

                ck0 = registers.ck();
            }

            if !registers.acc_complexity(instr.complexity()) {
                #[cfg(feature = "log")]
                eprintln!("complexity overflow");
                return None;
            }
            match next {
                ExecStep::Stop => {
                    #[cfg(feature = "log")]
                    {
                        let c = if registers.ck().is_ok() { g } else { r };
                        eprintln!("execution stopped; {d}CK {z}{c}{}{z}", registers.ck());
                    }
                    return None;
                }
                ExecStep::FailHalt => {
                    let _ = registers.fail_ck();
                    #[cfg(feature = "log")]
                    eprintln!("halting, {d}CK{z} is set to {r}false{z}");
                    return None;
                }
                ExecStep::Next => {
                    #[cfg(feature = "log")]
                    eprintln!();
                    continue;
                }
                ExecStep::FailContinue => {
                    if registers.fail_ck() {
                        #[cfg(feature = "log")]
                        eprintln!(
                            "halting, {d}CK{z} is set to {r}false{z} and {d}ch{z} is {r}true{z}"
                        );
                        return None;
                    }
                    #[cfg(feature = "log")]
                    eprintln!("failing, {d}CK{z} is set to {r}false{z}");
                    continue;
                }
                ExecStep::Jump(pos) => {
                    #[cfg(feature = "log")]
                    eprintln!("{}", pos);
                    if marshaller.seek(pos).is_err() {
                        let _ = registers.fail_ck();
                        #[cfg(feature = "log")]
                        eprintln!(
                            "jump to non-existing offset; halting, {d}CK{z} is set to {r}fail{z}"
                        );
                        return None;
                    }
                }
                ExecStep::Call(site) => {
                    #[cfg(feature = "log")]
                    eprintln!("{}", site);
                    return Some(site.into());
                }
            }
        }

        None
    }
}
