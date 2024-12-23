# AluVM rust implementation

![Build](https://github.com/AluVM/aluvm/workflows/Build/badge.svg)
![Tests](https://github.com/AluVM/aluvm/workflows/Tests/badge.svg)
![Lints](https://github.com/AluVM/aluvm/workflows/Lints/badge.svg)
[![codecov](https://codecov.io/gh/AluVM/aluvm/branch/master/graph/badge.svg)](https://codecov.io/gh/AluVM/aluvm)

[![crates.io](https://img.shields.io/crates/v/aluvm)](https://crates.io/crates/aluvm)
[![Docs](https://docs.rs/aluvm/badge.svg)](https://docs.rs/aluvm)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2 licensed](https://img.shields.io/crates/l/aluvm)](./LICENSE)

Rust implementation of AluVM (arithmetic logic unit virtual machine).

AluVM is a pure functional register-based highly deterministic & exception-less instruction set
architecture (ISA) and virtual machine (VM). The AluVM ISA can be extended by an environment running
the virtual machine (runtime environment), providing ability to load data to the VM registers and
support application-specific instructions (like SIMD).

The main purpose for ALuVM is to be used in distributed systems whether robustness,
platform-independent determinism are more important than the speed of computation. The main area of
AluVM applications (using appropriate ISA extensions) is blockchain environments, consensus-critical
computations, edge computing, multiparty computing (including deterministic machine learning),
client-side-validation, sandboxed computing and genetic algorithms.

For more details on AluVM, please check [the specification][AluVM], watch detailed presentation
on [YouTube] or check [slides] from the presentation.

## Design

The robustness lies at the very core of AluVM. It is designed to avoid any undefined behaviour.
Specifically,

* All registers may be in the undefined state;
* Impossible/incorrect operations put destination register into a special *undefined state*;
* Code always extended to 2^16 bytes with zeros, which corresponds to “set st0 register to false and
  stop execution” op-code;
* There are no invalid jump operations;
* There are no invalid instructions;
* Cycles & jumps are counted with 2^16 limit (bounded-time execution);
* No ambiguity: any two distinct byte strings always represent strictly distinct programs;
* Code and embedded data signing;
* Code commits to the used ISA extensions;
* Libraries identified by their hashes;
* Code does not run if not all libraries are present.

![Comparison table](doc/comparison.png)

## Instruction Set Architecture

![Instruction set architecture](doc/isa.png)

## History

- The need for AluVM recognized as a part of RGB project in Mar, the 24 & 31st, 2021 (see developers
  call <https://youtu.be/JmKNyOMv68I>)
- Concept was presented on 19th of May 2021([check the recoding](https://youtu.be/Mma0oyiVbSE))
- v0.1 release of Rust AluVM implementation on the 28th of May 2021
  ([ISA & API docs](https://docs.rs/aluvm/0.1.0/alure/))
- v0.2 release with multiple enhancements on the 9 Jun
  2021 ([ISA & API docs](https://docs.rs/aluvm/0.2.1/aluvm/)) – see presentation on [YouTube] or
  read [slides]
- At the end of 2024 v0.12 became a complete re-write, abstracting most of the instructions into a
  ISA extensions. The remaining core of AluVM become zk-STARK and zk-STARK-compatible, so a
  dedicated ISA extensions can be used to create fully arithmetized applications.

[AluVM]: https://github.com/AluVM/aluvm-spec

[YouTube]: https://www.youtube.com/watch?v=brfWta7XXFQ

[slides]: https://github.com/LNP-BP/presentations/blob/master/Presentation%20slides/AluVM.pdf
