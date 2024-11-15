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

/// Macro compiler for AluVM assembler.
///
/// # Example
///
/// ```
/// use aluvm::isa::Instr;
/// use aluvm::regs::Status;
/// use aluvm::{aluasm, Lib, LibId, LibSite, Vm};
///
/// let code = aluasm! {
///     nop                ;
///     put     CK, :ok    ;
///     chk                ;
/// };
///
/// let lib = Lib::assemble::<Instr<LibId>>(&code).unwrap();
/// let mut vm = Vm::<Instr<LibId>>::new();
/// match vm.exec(LibSite::new(lib.lib_id(), 0), &(), |_| Some(&lib)) {
///     Status::Ok => println!("success"),
///     Status::Fail => println!("failure"),
/// }
/// ```
#[macro_export]
macro_rules! aluasm {
    ($( $tt:tt )+) => {{ #[allow(unused_imports)] {
        use $crate::isa::{Instr, CtrlInstr,  ReservedInstr};
        $crate::aluasm_isa! { $( $tt )+ }
    } }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! aluasm_isa {
    ($( $tt:tt )+) => {{
        let mut code: Vec<Instr<$crate::LibId>> = vec![];
        #[allow(unreachable_code)] {
            $crate::aluasm_inner! { code => $( $tt )+ }
        }
        code
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! aluasm_inner {
    // end of program
    { $code:ident => } => { };
    // no operands
    { $code:ident => $op:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // special type
    { $code:ident => $op:ident $reg:ident, :$val:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $reg, :$val });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operand is an external jump to a named location in library literal
    { $code:ident => $op:ident $arg:literal @ $lib:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg @ $lib });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operand is an external jump to a position
    { $code:ident => $op:ident $arg:literal @ $lib:literal #h ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg @ $lib #h });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operand is an external jump to a named location in named library
    { $code:ident => $op:ident $arg:ident @ $lib:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg @ $lib });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operands are all literals
    { $code:ident => $op:ident $( $arg:literal ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operands are all idents
    { $code:ident => $op:ident $( $arg:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operands are all local registries
    { $code:ident => $op:ident $( $arg:ident : $idx:literal ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg : $idx  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operands are all argument registries
    { $code:ident => $op:ident $( $arg:ident : $idx:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg : $idx  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operands are all saved registries
    { $code:ident => $op:ident $( $arg:ident . $idx:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg . $idx  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    // operands are different types of registries
    { $code:ident => $op:ident $arg:ident . $idx:literal, $( $args:ident : $idxs:literal ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg . $idx, $( $args : $idxs  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:ident . $idx:literal, $( $args:ident : $idxs:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg . $idx, $( $args : $idxs  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:ident : $idx:literal, $( $args:ident . $idxs:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg : $idx, $( $args . $idxs  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:ident : $idx:ident, $( $args:ident . $idxs:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg : $idx, $( $args . $idxs  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
}

#[macro_export]
macro_rules! from_hex {
    ($ty:ty, $val:literal) => {
        $ty::from_str_radix(&stringify!($pos).expect("invalid hexadecimal literal"))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! instr {
    (nop) => {
        Instr::Ctrl(CtrlInstr::Nop)
    };
    (chk) => {
        Instr::Ctrl(CtrlInstr::Chk)
    };
    (not CO) => {
        Instr::Ctrl(CtrlInstr::NotCo)
    };
    (put CK, :fail) => {
        Instr::Ctrl(CtrlInstr::FailCk)
    };
    (put CK, :ok) => {
        Instr::Ctrl(CtrlInstr::RsetCk)
    };
    (ret) => {
        Instr::Ctrl(CtrlInstr::Ret)
    };
    (stop) => {
        Instr::Ctrl(CtrlInstr::Stop)
    };

    // Jumps
    (jmp $pos:literal) => {
        Instr::Ctrl(CtrlInstr::Jmp { pos: $pos })
    };
    (jmp $pos:literal #h) => {
        Instr::Ctrl(CtrlInstr::Jmp { pos: from_hex!(u16, $pos) })
    };
    (jif CO, $pos:literal) => {
        Instr::Ctrl(CtrlInstr::JiNe { pos: $pos })
    };
    (jif CO, $pos:literal #h) => {
        Instr::Ctrl(CtrlInstr::JiNe { pos: from_hex!(u16, $pos) })
    };
    (jif CK, $pos:literal) => {
        Instr::Ctrl(CtrlInstr::JiFail { pos: $pos })
    };
    (jif CK, $pos:literal #h) => {
        Instr::Ctrl(CtrlInstr::JiFail { pos: from_hex!(u16, $pos) })
    };
    (jif +$shift:literal) => {
        Instr::Ctrl(CtrlInstr::Sh { shift: $shift })
    };
    (jif +$shift:literal #h) => {
        Instr::Ctrl(CtrlInstr::Sh { shift: from_hex!(i8, $shift) })
    };
    (jif -$shift:literal) => {
        Instr::Ctrl(CtrlInstr::Sh { shift: $shift })
    };
    (jif -$shift:literal #h) => {
        Instr::Ctrl(CtrlInstr::Sh { shift: from_hex!(i8, $shift) })
    };
    (jif CO, +$shift:literal) => {
        Instr::Ctrl(CtrlInstr::ShNe { shift: $shift })
    };
    (jif CO, +$shift:literal #h) => {
        Instr::Ctrl(CtrlInstr::ShNe { shift: from_hex!(i8, $shift) })
    };
    (jif CK, -$shift:literal) => {
        Instr::Ctrl(CtrlInstr::ShFail { shift: $shift })
    };
    (jif CK, -$shift:literal #h) => {
        Instr::Ctrl(CtrlInstr::ShFail { shift: from_hex!(i8, $shift) })
    };

    // Calls
    (jmp $lib:ident @ $pos:literal) => {
        Instr::Ctrl(CtrlInstr::Exec { site: $crate::Site::new($lib, $pos) })
    };
    (jmp $lib:ident @ $pos:literal #h) => {
        Instr::Ctrl(CtrlInstr::Exec { site: $crate::Site::new($lib, from_hex!(u16, $pos)) })
    };
    (call $lib:ident @ $pos:literal) => {
        Instr::Ctrl(CtrlInstr::Call { site: $crate::Site::new($lib, $pos) })
    };
    (call $lib:ident @ $pos:literal #h) => {
        Instr::Ctrl(CtrlInstr::Call { site: $crate::Site::new($lib, from_hex!(u16, $pos)) })
    };
    (call $pos:literal) => {
        Instr::Ctrl(CtrlInstr::Fn { pos: $pos })
    };
    (call $pos:literal #h) => {
        Instr::Ctrl(CtrlInstr::Fn { pos: from_hex!(u16, $pos) })
    };

    { $($tt:tt)+ } => {
        Instr::Reserved(isa_instr! { $( $tt )+ })
    };
}
