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

#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    dead_code,
    // missing_docs
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! Rust implementation of AluVM (arithmetic logic unit virtual machine) and assembler from Alu
//! Assembly language into bytecode.
//!
//! AluVM is a pure functional register-based highly deterministic & exception-less instruction set
//! architecture (ISA) and virtual machine (VM) without random memory access, capable of performing
//! arithmetic operations, including operations on elliptic curves. The AluVM ISA can be extended by
//! the environment running the virtual machine (host environment), providing ability to load data
//! to the VM registers and support application-specific instructions (like SIMD).
//!
//! The main purpose for ALuVM is to be used in distributed systems whether robustness,
//! platform-independent determinism are more important than the speed of computation. The main area
//! of AluVM applications (using appropriate ISA extensions) is blockchain environments,
//! consensus-critical computations, multiparty computing (including deterministic machine
//! learning), client-side-validation, sandboxed Internet2 computing and genetic algorithms.
//!
//! For more details on AluVM, please check [the specification][AluVM]
//!
//!
//! ## Design
//!
//! The robusness lies at the very core of AluVM. It is designed to avoid any
//! undefined behaviour. Specifically,
//! * All registers may be in the undefined statel
//! * Impossible/incorrect operations put destination register into a special *undefined state*;
//! * Code always extended to 2^16 bytes with zeros, which corresponds to “set st0 register to false
//!   and stop execution” op-code;
//! * There are no invalid jump operations;
//! * There are no invalid instructions;
//! * Cycles & jumps are counted with 2^16 limit (bounded-time execution);
//! * No ambiguity: any two distinct byte strings always represent strictly distinct programs;
//! * Code is always signed;
//! * Data segment is always signed;
//! * Code commits to the used ISA extensions;
//! * Libraries identified by the signature;
//! * Code does not run if not all libraries are present;
//!
//! ![Comparison table](doc/comparison.png)
//!
//!
//! ## Instruction Set Architecture
//!
//! ![Instruction set architecture](doc/isa.png)
//!
//! ### Instruction opcodes
//!
//! You will find all opcode implementation details documented in [`isa::Instr`] API docs.
//!
//! - RISC: only 256 instructions
//! - 3 families of core instructions:
//!   * Control flow
//!   * Data load / movement between registers
//!   * ALU (including cryptography)
//! - Extensible with ISA extensions: 127 of the operations are reserved for extensions
//!   * More cryptography
//!   * Custom data I/O (blockchain, LN, client-side-validation)
//!   * Genetic algorithms / code self-modification
//!   
//! The arithmetic ISA is designed with strong robustness goals:
//! - Impossible arithmetic operation (0/0, Inf/inf) always sets the destination register into
//!   undefined state (unlike NaN in IEEE-754 it has only a single unique value)
//! - Operation resulting in the value which can't fit the bit dimensions under a used encoding,
//!   including representation of infinity for integer encodings (x/0 if x != 0) results in:
//!   * for float underflows, subnormally encoded number,
//!   * for x/0 if x != 0 on float numbers, ±Inf float value,
//!   * for overflows in integer checked operations and floats: undefined value, setting st0 to
//!     false,
//!   * for overflows in integer wrapped operations, modulo division on the maximum register value
//!
//! Most of the arithmetic operations has to be provided with flags specifying which of the encoding
//! and exception handling should be used:
//! * Integer encodings has two flags:
//!   - one for signed/unsigned variant of the encoding
//!   - one for checked or wrapped variant of exception handling
//! * Float encoding has 4 variants of rounding, matching IEEE-754 options
//!
//! Thus, many arithmetic instructions have 8 variants, indicating the used encoding (unsigned,
//! signed integer or float) and operation behavior in situation when resulting value does not fit
//! into the register (overflow or wrap for integers and one of four rounding options for floats).
//!
//! Check [the specification][AluVM] for the details.
//!
//! ### Registers
//!
//! **ALU registers:** 8 blocks of 32 registers
//! - Integer arithmetic (A-registers) blocks: 8, 16, 32, 64, 128, 256, 512, 1024 bits
//! - Float arithmetic (F-registers) blocks:
//!   * IEEE: binary-half, single, double, quad, oct precision
//!   * IEEE extension: 80-bit X87 register
//!   * BFloat16 register, used in Machine learning
//! - Cryptographic operations (R-registers) blocks: 128, 160, 256, 512, 1024, 2048, 4096, 8192 bits
//! - String registers (S-registers): 1 block of 256 registers, 64kb each
//!
//! **Control flow registers:**
//! - Status (st0), boolean (one bit)
//! - Cycle counter (cy0), 16 bits
//! - Instruction complexity accumulator (ca0), 16 bits
//! - Call stack register (cs0), 3*2^16 bits (192kB block)
//! - Call stack pointer register (cp0), 16 bits
//!
//! [AluVM]: https://github.com/AluVM/aluvm-spec

#![allow(clippy::bool_assert_comparison)]

// TODO: Extend the list of features not compatible with zk-aluvm as they appear.
#[cfg(all(feature = "zk-aluvm", any(feature = "A64", feature = "STR")))]
compile_error!("zk-AluVM is incompatible with any ISA extensions other then GFA");

extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[macro_use]
extern crate commit_verify;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod core;
#[macro_use]
pub mod isa;
mod library;
mod vm;
#[cfg(feature = "stl")]
pub mod stl;

pub mod regs {
    pub use crate::core::{IdxA, IdxAl, Reg, RegA, Status, A, CALL_STACK_SIZE_MAX};
}

pub use isa::{IsaId, ISA_ALU128, ISA_ALU64, ISA_AN, ISA_ID_MAX_LEN};
#[cfg(feature = "armor")]
pub use library::armor::LibArmorError;
pub use library::{Lib, LibId, LibSite};
#[doc(hidden)]
pub use paste::paste;
pub use vm::Vm;

#[cfg(feature = "GFA")]
pub use self::core::gfa;
pub use self::core::{Core, CoreConfig, Site, SiteId};

pub const LIB_NAME_ALUVM: &str = "AluVM";
