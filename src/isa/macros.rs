// AluVM Assembler
// To find more on AluVM please check <https://www.aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2024 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Institute. All rights reserved.
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
/// # use aluvm::aluasm;
/// # use aluvm::Vm;
/// # use aluvm::library::{Lib, LibSite};
/// # use aluvm::isa::{Instr, InstructionSet, ReservedOp};
///
/// let code = aluasm! {
///         clr     r1024[5]                        ;
///         put     a16[8],378                      ;
///         putif   r128[5],0xaf67937b5498dc        ;
///         swp     a8[1],a8[2]                     ;
///         swp     f256[8],f256[7]                 ;
///         dup     a256[1],a256[7]                 ;
///         mov     a16[1],a16[2]                   ;
///         mov     r256[8],r256[7]                 ;
///         cpy     a256[1],a256[7]                 ;
///         ret                                     ;
///         jmp     0                               ;
/// };
///
/// let lib = Lib::assemble(&code, Instr::<ReservedOp>::isa_ids()).unwrap();
/// let mut vm = Vm::<Instr>::new();
/// match vm.exec(LibSite::default(), |_| Some(&lib), &()) {
///     true => println!("success"),
///     false => println!("failure"),
/// }
/// ```
#[macro_export]
macro_rules! aluasm {
    ($( $tt:tt )+) => {{ #[allow(unused_imports)] {
        use ::aluvm::isa::ReservedOp;
        $crate::aluasm_isa! { ReservedOp => $( $tt )+ }
    } }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! aluasm_isa {
    ($isa:ty => $( $tt:tt )+) => {{
        use ::std::boxed::Box;

        use ::aluvm::isa::{
            ArithmeticOp, BitwiseOp, BytesOp, CmpOp, ControlFlowOp, DigestOp, ExtendFlag, FloatEqFlag, Instr, IntFlags,
            MergeFlag, MoveOp, PutOp, RoundingFlag, Secp256k1Op, SignFlag, NoneEqFlag
        };
        use ::aluvm::reg::{
            Reg16, Reg32, Reg8, RegA, RegA2, RegAR, RegBlockAFR, RegBlockAR, RegF, RegR, RegS,
            NumericRegister,
        };
        use ::aluvm::library::LibSite;
        use ::aluvm::data::{ByteStr, Number, MaybeNumber, Step};

        let mut code: Vec<Instr<$isa>> = vec![];
        #[allow(unreachable_code)] {
            $crate::aluasm_inner! { code => $( $tt )+ }
        }
        code
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! aluasm_inner {
    { $code:ident => } => { };
    { $code:ident => $op:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:literal @ $lib:literal ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg @ $lib });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:ident @ $lib:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg @ $lib });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:literal ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident . $flag:ident $( $arg:ident ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op . $flag $( $arg ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $( $arg [ $idx ]  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident . $flag:ident $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op . $flag $( $arg [ $idx ]  ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arglit:literal, $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arglit, $( $arg [ $idx ] ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arglit:ident, $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arglit, $( $arg [ $idx ] ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident . $flag:ident $arglit:literal, $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op . $flag $arglit, $( $arg [ $idx ] ),+ });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arglit1:literal, $arglit2:literal, $arg:ident [ $idx:literal ] ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arglit1, $arglit2, $arg [ $idx ] });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident . $flag:ident $arglit1:literal, $arglit2:literal $arg:ident [ $idx:literal ] ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op . $flag $arglit1, $arglit2, $arg [ $idx ] });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:ident [ $idx:literal ] , $arglit:literal ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg [ $idx ] , $arglit });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arg:ident [ $idx:literal ] , $arglit:ident ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op $arg [ $idx ] , $arglit });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident . $flag:ident $arg:ident [ $idx:literal ], $arglit:expr ; $($tt:tt)* } => {
        $code.push($crate::instr!{ $op . $flag $arg [ $idx ], $arglit });
        $crate::aluasm_inner! { $code => $( $tt )* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! instr {
    (fail) => {
        Instr::ControlFlow(ControlFlowOp::Fail)
    };
    (test) => {
        Instr::ControlFlow(ControlFlowOp::Test)
    };
    (jmp $offset:literal) => {
        Instr::ControlFlow(ControlFlowOp::Jmp($offset))
    };
    (jmp $offset:ident) => {
        Instr::ControlFlow(ControlFlowOp::Jmp($offset))
    };
    (jif $offset:literal) => {
        Instr::ControlFlow(ControlFlowOp::Jif($offset))
    };
    (jif $offset:ident) => {
        Instr::ControlFlow(ControlFlowOp::Jif($offset))
    };
    (routine $offset:literal) => {
        Instr::ControlFlow(ControlFlowOp::Reutine($offset))
    };
    (routine $offset:ident) => {
        Instr::ControlFlow(ControlFlowOp::Reutine($offset))
    };
    (call $offset:literal @ $lib:literal) => {
        Instr::ControlFlow(ControlFlowOp::Call(LibSite::with(
            $offset,
            $lib.parse().expect("wrong library reference"),
        )))
    };
    (call $offset:ident @ $lib:ident) => {
        Instr::ControlFlow(ControlFlowOp::Call(LibSite::with(
            $offset,
            $lib
        )))
    };
    (exec $offset:literal @ $lib:literal) => {
        Instr::ControlFlow(ControlFlowOp::Exec(LibSite::with(
            $offset,
            $lib.parse().expect("wrong library reference"),
        )))
    };
    (exec $offset:ident @ $lib:ident) => {
        Instr::ControlFlow(ControlFlowOp::Exec(LibSite::with(
            $offset,
            $lib
        )))
    };
    (ret) => {
        Instr::ControlFlow(ControlFlowOp::Ret)
    };

    (clr $reg:ident[$idx:literal]) => {
        Instr::Put($crate::_reg_sfx!(PutOp, Clr, $reg)(
            $crate::_reg_ty!(Reg, $reg),
            $crate::_reg_idx!($idx),
        ))
    };

    (extr s16[$idx:literal], $reg:ident[$reg_idx:literal], a16[$offset_idx:literal]) => {
        Instr::Bytes(BytesOp::Extr(
            RegS::from($idx),
            $crate::_reg_tyar!($reg),
            $crate::_reg_idx16!($reg_idx),
            $crate::_reg_idx16!($offset_idx),
        ))
    };
    (inj s16[$idx:literal], $reg:ident[$reg_idx:literal], a16[$offset_idx:literal]) => {
        Instr::Bytes(BytesOp::Inj(
            RegS::from($idx),
            $crate::_reg_tyar!($reg),
            $crate::_reg_idx16!($reg_idx),
            $crate::_reg_idx16!($offset_idx),
        ))
    };
    (put s16[$idx:literal], $val:ident) => {{
        Instr::Bytes(BytesOp::Put(RegS::from($idx), Box::new(ByteStr::with(&$val)), false))
    }};
    (put s16[$idx:literal], $val:literal) => {{
        Instr::Bytes(BytesOp::Put(RegS::from($idx), Box::new(ByteStr::with(&$val)), false))
    }};
    (fill.e s16[$idx0:literal],a16[$idx1:literal],a16[$idx2:literal],a8[$idx3:literal]) => {{
        Instr::Bytes(BytesOp::Fill(
            RegS::from($idx0),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
            $crate::_reg_idx!($idx3),
            ExtendFlag::Extend,
        ))
    }};
    (fill.f s16[$idx0:literal],a16[$idx1:literal],a16[$idx2:literal],a8[$idx3:literal]) => {{
        Instr::Bytes(BytesOp::Fill(
            RegS::from($idx0),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
            $crate::_reg_idx!($idx3),
            ExtendFlag::Fail,
        ))
    }};
    (len s16[$s_idx:literal], $rega:ident[$rega_idx:literal]) => {{
        Instr::Bytes(BytesOp::Len(
            RegS::from($s_idx),
            $crate::_reg_tya!(Reg, $rega),
            $crate::_reg_idx!($rega_idx),
        ))
    }};
    (cnt s16[$s_idx:literal],a8[$byte_idx:literal],a16[$dst_idx:literal]) => {{
        Instr::Bytes(BytesOp::Cnt(
            RegS::from($s_idx),
            $crate::_reg_idx16!($byte_idx),
            $crate::_reg_idx16!($dst_idx),
        ))
    }};
    (
        con s16[$src1_idx:literal],s16[$src2_idx:literal],a16[$frag_idx:literal],a16[$offset_dst_idx:literal],a16[$len_dst_idx:literal]
    ) => {{
        Instr::Bytes(BytesOp::Con(
            RegS::from($src1_idx),
            RegS::from($src2_idx),
            $crate::_reg_idx!($frag_idx),
            $crate::_reg_idx!($offset_dst_idx),
            $crate::_reg_idx!($len_dst_idx),
        ))
    }};
    (find s16[$str_idx:literal],s16[$fragment_idx:literal],a16[$should_be_0:literal]) => {{
        assert_eq!(0, $should_be_0);
        Instr::Bytes(BytesOp::Find(RegS::from($str_idx), RegS::from($fragment_idx)))
    }};
    (rev s16[$src_idx:literal],s16[$dst_idx:literal]) => {{
        Instr::Bytes(BytesOp::Rev(RegS::from($src_idx), RegS::from($dst_idx)))
    }};
    (put $reg:ident[$idx:literal], $val:literal) => {{
        let s = stringify!($val);
        let mut num = s.parse::<MaybeNumber>().expect(&format!("invalid number literal `{}`", s));
        let reg = $crate::_reg_ty!(Reg, $reg);
        num.reshape(reg.layout());
        Instr::Put($crate::_reg_sfx!(PutOp, Put, $reg)(reg, $crate::_reg_idx!($idx), Box::new(num)))
    }};
    (put $reg:ident[$idx:literal], $val:ident) => {{
        let mut num = MaybeNumber::from($val);
        let reg = $crate::_reg_ty!(Reg, $reg);
        num.reshape(reg.layout());
        Instr::Put($crate::_reg_sfx!(PutOp, Put, $reg)(reg, $crate::_reg_idx!($idx), Box::new(num)))
    }};
    (putif $reg:ident[$idx:literal], $val:literal) => {{
        let s = stringify!($val);
        let mut num = s.parse::<MaybeNumber>().expect(&format!("invalid number literal `{}`", s));
        let reg = $crate::_reg_ty!(Reg, $reg);
        num.reshape(reg.layout());
        Instr::Put($crate::_reg_sfx!(PutOp, PutIf, $reg)(
            reg,
            $crate::_reg_idx!($idx),
            Box::new(num),
        ))
    }};
    (putif $reg:ident[$idx:literal], $val:ident) => {{
        let mut num = MaybeNumber::from($val);
        let reg = $crate::_reg_ty!(Reg, $reg);
        num.reshape(reg.layout());
        Instr::Put($crate::_reg_sfx!(PutOp, PutIf, $reg)(
            reg,
            $crate::_reg_idx!($idx),
            Box::new(num),
        ))
    }};

    (swp $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $reg2) {
            panic!("Swap operation must be performed between registers of the same type");
        }
        Instr::Move($crate::_reg_sfx!(MoveOp, Swp, $reg1)(
            $crate::_reg_ty!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (mov $src_reg:ident[$src_idx:literal], $dst_reg:ident[$dst_idx:literal]) => {{
        if $crate::_reg_ty!(Reg, $src_reg) != $crate::_reg_ty!(Reg, $dst_reg) {
            panic!("Move operation must be performed between registers of the same type");
        }
        Instr::Move($crate::_reg_sfx!(MoveOp, Mov, $src_reg)(
            $crate::_reg_ty!(Reg, $src_reg),
            $crate::_reg_idx!($src_idx),
            $crate::_reg_idx!($dst_idx),
        ))
    }};
    (dup $src_reg:ident[$src_idx:literal], $dst_reg:ident[$dst_idx:literal]) => {{
        if $crate::_reg_ty!(Reg, $src_reg) != $crate::_reg_ty!(Reg, $dst_reg) {
            panic!("Dup operation must be performed between registers of the same type");
        }
        Instr::Move($crate::_reg_sfx!(MoveOp, Dup, $src_reg)(
            $crate::_reg_ty!(Reg, $src_reg),
            $crate::_reg_idx!($src_idx),
            $crate::_reg_idx!($dst_idx),
        ))
    }};
    (cpy $src_reg:ident[$src_idx:literal], $dst_reg:ident[$dst_idx:literal]) => {{
        if $crate::_reg_ty!(Reg, $src_reg) != $crate::_reg_ty!(Reg, $dst_reg) {
            panic!("Copy operation must be performed between registers of the same type");
        }
        Instr::Move($crate::_reg_sfx!(MoveOp, Cpy, $src_reg)(
            $crate::_reg_ty!(Reg, $src_reg),
            $crate::_reg_idx!($src_idx),
            $crate::_reg_ty!(Reg, $dst_reg),
            $crate::_reg_idx!($dst_idx),
        ))
    }};
    (cnv $src_reg:ident[$src_idx:literal], $dst_reg:ident[$dst_idx:literal]) => {{
        match ($crate::_reg_block!($src_reg), $crate::_reg_block!($dst_reg)) {
            (RegBlockAFR::A, RegBlockAFR::F) => Instr::Move(MoveOp::CnvAF(
                $crate::_reg_tya!(Reg, $src_reg),
                $crate::_reg_idx!($src_idx),
                $crate::_reg_tyf!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::F, RegBlockAFR::A) => Instr::Move(MoveOp::CnvFA(
                $crate::_reg_tyf!(Reg, $src_reg),
                $crate::_reg_idx!($src_idx),
                $crate::_reg_tya!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Move(MoveOp::CnvA(
                $crate::_reg_tya!(Reg, $src_reg),
                $crate::_reg_idx!($src_idx),
                $crate::_reg_tya!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Move(MoveOp::CnvF(
                $crate::_reg_tyf!(Reg, $src_reg),
                $crate::_reg_idx!($src_idx),
                $crate::_reg_tyf!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
            )),
            (_, _) => panic!("Conversion operation between unsupported register types"),
        }
    }};
    (spy $src_reg:ident[$src_idx:literal], $dst_reg:ident[$dst_idx:literal]) => {{
        match ($crate::_reg_block!($src_reg), $crate::_reg_block!($dst_reg)) {
            (RegBlockAFR::A, RegBlockAFR::R) => Instr::Move(MoveOp::SpyAR(
                $crate::_reg_tya!(Reg, $src_reg),
                $crate::_reg_idx!($src_idx),
                $crate::_reg_tyr!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::R, RegBlockAFR::A) => Instr::Move(MoveOp::SpyAR(
                $crate::_reg_tya!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
                $crate::_reg_tyr!(Reg, $src_reg),
                $crate::_reg_idx!($src_idx),
            )),
            (_, _) => {
                panic!("Swap-conversion operation is supported only between A and R registers")
            }
        }
    }};

    (gt $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        if $crate::_reg_block!($reg1) != RegBlockAFR::R {
            panic!("`gt` operation for arithmetic registers requires suffix");
        }
        Instr::Cmp(CmpOp::GtR(
            $crate::_reg_tyr!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (gt.u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            SignFlag::Unsigned,
            $crate::_reg_tya!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (gt.s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            SignFlag::Signed,
            $crate::_reg_tya!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (gt.e $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtF(
            FloatEqFlag::Exact,
            $crate::_reg_tyf!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (gt.r $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtF(
            FloatEqFlag::Rounding,
            $crate::_reg_tyf!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (lt $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        if $crate::_reg_block!($reg1) != RegBlockAFR::R {
            panic!("`lt` operation for arithmetic registers requires suffix");
        }
        Instr::Cmp(CmpOp::LtR(
            $crate::_reg_tyr!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (lt.u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            SignFlag::Unsigned,
            $crate::_reg_tya!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (lt.s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            SignFlag::Signed,
            $crate::_reg_tya!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (lt.e $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtF(
            FloatEqFlag::Exact,
            $crate::_reg_tyf!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (lt.r $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtF(
            FloatEqFlag::Rounding,
            $crate::_reg_tyf!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (eq.e $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!(
                "Equivalence check must be performed between registers of the same type and size"
            );
        }
        match $crate::_reg_block!($reg1) {
            RegBlockAFR::A => Instr::Cmp(CmpOp::EqA(
                NoneEqFlag::Equal,
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($idx2),
            )),
            RegBlockAFR::R => Instr::Cmp(CmpOp::EqR(
                NoneEqFlag::Equal,
                $crate::_reg_tyr!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($idx2),
            )),
            RegBlockAFR::F => Instr::Cmp(CmpOp::EqF(
                FloatEqFlag::Exact,
                $crate::_reg_tyf!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($idx2),
            )),
        }
    }};
    (eq.n $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!(
                "Equivalence check must be performed between registers of the same type and size"
            );
        }
        match $crate::_reg_block!($reg1) {
            RegBlockAFR::A => Instr::Cmp(CmpOp::EqA(
                NoneEqFlag::NonEqual,
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($idx2),
            )),
            RegBlockAFR::R => Instr::Cmp(CmpOp::EqR(
                NoneEqFlag::NonEqual,
                $crate::_reg_tyr!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($idx2),
            )),
            _ => panic!("Wrong registers for `eq` operation"),
        }
    }};
    (eq.r $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if $crate::_reg_block!($reg1) != $crate::_reg_block!($reg2) {
            panic!(
                "Equivalence check must be performed between registers of the same type and size"
            );
        }
        Instr::Cmp(CmpOp::EqF(
            FloatEqFlag::Rounding,
            $crate::_reg_tyf!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (eq s16[$idx1:literal],s16[$idx2:literal]) => {{
        Instr::Bytes(BytesOp::Eq(RegS::from($idx1), RegS::from($idx2)))
    }};
    (ifn $reg:ident[$idx:literal]) => {
        match $crate::_reg_block!($reg) {
            RegBlockAFR::A => {
                Instr::Cmp(CmpOp::IfNA($crate::_reg_tya!(Reg, $reg), $crate::_reg_idx!($idx)))
            }
            RegBlockAFR::R => {
                Instr::Cmp(CmpOp::IfNR($crate::_reg_tyr!(Reg, $reg), $crate::_reg_idx!($idx)))
            }
            _ => panic!("Wrong registers for `ifn` operation"),
        }
    };
    (ifz $reg:ident[$idx:literal]) => {
        match $crate::_reg_block!($reg) {
            RegBlockAFR::A => {
                Instr::Cmp(CmpOp::IfZA($crate::_reg_tya!(Reg, $reg), $crate::_reg_idx!($idx)))
            }
            RegBlockAFR::R => {
                Instr::Cmp(CmpOp::IfZR($crate::_reg_tyr!(Reg, $reg), $crate::_reg_idx!($idx)))
            }
            _ => panic!("Wrong registers for `ifz` operation"),
        }
    };
    (st. $flag:ident $reg:ident[$idx:literal]) => {
        Instr::Cmp(CmpOp::St(
            $crate::_merge_flag!($flag),
            $crate::_reg_tya!(Reg, $reg),
            $crate::_reg_idx8!($idx),
        ))
    };
    (inv st0) => {
        Instr::Cmp(CmpOp::StInv)
    };

    (add. $flag:ident $reg1:ident[$idx1:literal], $dst_reg:ident[$dst_idx:literal]) => {
        match ($crate::_reg_block!($reg1), $crate::_reg_block!($dst_reg)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::AddA(
                $crate::_int_flags!($flag),
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::AddF(
                $crate::_rounding_flag!($flag),
                $crate::_reg_tyf!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (a, b) if a == b => panic!("addition requires integer or float registers"),
            (_, _) if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $dst_reg) => {
                panic!("addition must be performed between registers of the same size")
            }
            (_, _) => panic!("addition must be performed between registers of the same type"),
        }
    };
    (sub. $flag:ident $reg1:ident[$idx1:literal], $dst_reg:ident[$dst_idx:literal]) => {
        match ($crate::_reg_block!($reg1), $crate::_reg_block!($dst_reg)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::SubA(
                $crate::_int_flags!($flag),
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::SubF(
                $crate::_rounding_flag!($flag),
                $crate::_reg_tyf!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (a, b) if a == b => panic!("subtraction requires integer or float registers"),
            (_, _) if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $dst_reg) => {
                panic!("subtraction must be performed between registers of the same size")
            }
            (_, _) => panic!("subtraction must be performed between registers of the same type"),
        }
    };
    (mul. $flag:ident $reg1:ident[$idx1:literal], $dst_reg:ident[$dst_idx:literal]) => {
        match ($crate::_reg_block!($reg1), $crate::_reg_block!($dst_reg)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::MulA(
                $crate::_int_flags!($flag),
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::MulF(
                $crate::_rounding_flag!($flag),
                $crate::_reg_tyf!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (a, b) if a == b => panic!("multiplication requires integer or float registers"),
            (_, _) if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $dst_reg) => {
                panic!("multiplication must be performed between registers of the same size")
            }
            (_, _) => panic!("multiplication must be performed between registers of the same type"),
        }
    };
    (div. $flag:ident $reg1:ident[$idx1:literal], $dst_reg:ident[$dst_idx:literal]) => {
        match ($crate::_reg_block!($reg1), $crate::_reg_block!($dst_reg)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::DivA(
                $crate::_int_flags!($flag),
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::DivF(
                $crate::_rounding_flag!($flag),
                $crate::_reg_tyf!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_idx!($dst_idx),
            )),
            (a, b) if a == b => panic!("division requires integer or float registers"),
            (_, _) if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $dst_reg) => {
                panic!("division must be performed between registers of the same size")
            }
            (_, _) => panic!("division must be performed between registers of the same type"),
        }
    };
    (rem $reg1:ident[$idx1:literal], $dst_reg:ident[$dst_idx:literal]) => {
        if $crate::_reg_block!($reg1) != RegBlockAFR::A
            || $crate::_reg_block!($dst_reg) != RegBlockAFR::A
        {
            panic!("modulo division must be performed only using integer arithmetic registers");
        } else {
            Instr::Arithmetic(ArithmeticOp::Rem(
                $crate::_reg_tya!(Reg, $reg1),
                $crate::_reg_idx!($idx1),
                $crate::_reg_tya!(Reg, $dst_reg),
                $crate::_reg_idx!($dst_idx),
            ))
        }
    };
    (inc $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            $crate::_reg_tya!(Reg, $reg),
            $crate::_reg_idx!($idx),
            Step::with(1),
        ))
    };
    (add $reg:ident[$idx:literal], $step:literal) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            $crate::_reg_tya!(Reg, $reg),
            $crate::_reg_idx!($idx),
            Step::with($step),
        ))
    };
    (dec $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            $crate::_reg_tya!(Reg, $reg),
            $crate::_reg_idx!($idx),
            Step::with(-1),
        ))
    };
    (sub $reg:ident[$idx:literal], $step:literal) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            $crate::_reg_tya!(Reg, $reg),
            $crate::_reg_idx!($idx),
            Step::with($step * -1),
        ))
    };
    (neg $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Neg(
            $crate::_reg_ty!(Reg, $reg).into(),
            $crate::_reg_idx16!($idx),
        ))
    };
    (abs $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Abs(
            $crate::_reg_ty!(Reg, $reg).into(),
            $crate::_reg_idx16!($idx),
        ))
    };

    (
        and
        $reg1:ident[$idx1:literal],
        $reg2:ident[$idx2:literal],
        $dst_reg:ident[$dst_idx:literal]
    ) => {
        if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $reg2)
            || $crate::_reg_ty!(Reg, $reg2) != $crate::_reg_ty!(Reg, $dst_reg)
        {
            panic!("`and` operation must use the same type of registers for all of its operands");
        } else if $crate::_reg_block!($reg1) != RegBlockAFR::A
            && $crate::_reg_block!($reg1) != RegBlockAFR::R
        {
            panic!("`and` operation requires integer arithmetic or generic registers");
        } else {
            Instr::Bitwise(BitwiseOp::And(
                $crate::_reg_ty!(Reg, $reg1).into(),
                $crate::_reg_idx16!($idx1),
                $crate::_reg_idx16!($idx2),
                $crate::_reg_idx16!($dst_idx),
            ))
        }
    };
    (
        or $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal], $dst_reg:ident[$dst_idx:literal]
    ) => {
        if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $reg2)
            || $crate::_reg_ty!(Reg, $reg2) != $crate::_reg_ty!(Reg, $dst_reg)
        {
            panic!("`or` operation must use the same type of registers for all of its operands");
        } else if $crate::_reg_block!($reg1) != RegBlockAFR::A
            && $crate::_reg_block!($reg1) != RegBlockAFR::R
        {
            panic!("`or` operation requires integer arithmetic or generic registers");
        } else {
            Instr::Bitwise(BitwiseOp::Or(
                $crate::_reg_ty!(Reg, $reg1).into(),
                $crate::_reg_idx16!($idx1),
                $crate::_reg_idx16!($idx2),
                $crate::_reg_idx16!($dst_idx),
            ))
        }
    };
    (
        xor
        $reg1:ident[$idx1:literal],
        $reg2:ident[$idx2:literal],
        $dst_reg:ident[$dst_idx:literal]
    ) => {
        if $crate::_reg_ty!(Reg, $reg1) != $crate::_reg_ty!(Reg, $reg2)
            || $crate::_reg_ty!(Reg, $reg2) != $crate::_reg_ty!(Reg, $dst_reg)
        {
            panic!("`xor` operation must use the same type of registers for all of its operands");
        } else if $crate::_reg_block!($reg1) != RegBlockAFR::A
            && $crate::_reg_block!($reg1) != RegBlockAFR::R
        {
            panic!("`xor` operation requires integer arithmetic or generic registers");
        } else {
            Instr::Bitwise(BitwiseOp::Xor(
                $crate::_reg_ty!(Reg, $reg1).into(),
                $crate::_reg_idx16!($idx1),
                $crate::_reg_idx16!($idx2),
                $crate::_reg_idx16!($dst_idx),
            ))
        }
    };
    (shl $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::Shl(
            $crate::_reg_tya2!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_ty!(Reg, $reg2).into(),
            $crate::_reg_idx!($idx2),
        ))
    };
    (shr.u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::ShrA(
            SignFlag::Unsigned,
            $crate::_reg_tya2!(Reg, $reg1),
            $crate::_reg_idx16!($idx1),
            $crate::_reg_ty!(Reg, $reg2),
            $crate::_reg_idx!($idx2),
        ))
    };
    (shr.s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::ShrA(
            SignFlag::Signed,
            $crate::_reg_tya2!(Reg, $reg1),
            $crate::_reg_idx16!($idx1),
            $crate::_reg_ty!(Reg, $reg2),
            $crate::_reg_idx!($idx2),
        ))
    };
    (shr $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        Instr::Bitwise(BitwiseOp::ShrR(
            $crate::_reg_tya2!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_ty!(Reg, $reg2),
            $crate::_reg_idx!($idx2),
        ))
    }};
    (scl $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::Scl(
            $crate::_reg_tya2!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_ty!(Reg, $reg2).into(),
            $crate::_reg_idx!($idx2),
        ))
    };
    (scr $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::Scr(
            $crate::_reg_tya2!(Reg, $reg1),
            $crate::_reg_idx!($idx1),
            $crate::_reg_ty!(Reg, $reg2).into(),
            $crate::_reg_idx!($idx2),
        ))
    };
    (rev $reg:ident[$idx:literal]) => {
        match $crate::_reg_block!($reg) {
            RegBlockAFR::A => Instr::Bitwise(BitwiseOp::RevA(
                $crate::_reg_tya!(Reg, $reg),
                $crate::_reg_idx!($idx),
            )),
            RegBlockAFR::R => Instr::Bitwise(BitwiseOp::RevR(
                $crate::_reg_tyr!(Reg, $reg),
                $crate::_reg_idx!($idx),
            )),
            _ => panic!("Wrong registers for `rev` operation"),
        }
    };

    (ripemd s16[$idx1:literal],r160[$idx2:literal]) => {
        Instr::Digest(DigestOp::Ripemd(RegS::from($idx1), $crate::_reg_idx16!($idx2)))
    };
    (sha2 s16[$idx1:literal],r256[$idx2:literal]) => {
        Instr::Digest(DigestOp::Sha256(RegS::from($idx1), $crate::_reg_idx16!($idx2)))
    };
    (blake3 s16[$idx1:literal],r256[$idx2:literal]) => {
        Instr::Digest(DigestOp::Blake3(RegS::from($idx1), $crate::_reg_idx16!($idx2)))
    };
    (sha2 s16[$idx1:literal],r512[$idx2:literal]) => {
        Instr::Digest(DigestOp::Sha512(RegS::from($idx1), $crate::_reg_idx16!($idx2)))
    };

    (secpgen $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if $crate::_reg_block!($reg1) != RegBlockAFR::R
            || $crate::_reg_block!($reg2) != RegBlockAFR::R
        {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Gen($crate::_reg_idx!($idx1), $crate::_reg_idx8!($idx2)))
        }
    };
    (
        secpmul
        $scalar_reg:ident[$scalar_idx:literal],
        $src_reg:ident[$src_idx:literal],
        $dst_reg:ident[$dst_idx:literal]
    ) => {
        if $crate::_reg_ty!(Reg, $src_reg) != $crate::_reg_ty!(Reg, $dst_reg) {
            panic!("ecmul instruction can be used only with registers of the same type");
        } else {
            Instr::Secp256k1(Secp256k1Op::Mul(
                $crate::_reg_block_ar!($scalar_reg),
                $crate::_reg_idx!($scalar_idx),
                $crate::_reg_idx!($src_idx),
                $crate::_reg_idx!($dst_idx),
            ))
        }
    };
    (secpadd $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if $crate::_reg_block!($reg1) != RegBlockAFR::R
            || $crate::_reg_block!($reg2) != RegBlockAFR::R
        {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Add($crate::_reg_idx!($idx1), $crate::_reg_idx8!($idx2)))
        }
    };
    (secpneg $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if $crate::_reg_block!($reg1) != RegBlockAFR::R
            || $crate::_reg_block!($reg2) != RegBlockAFR::R
        {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Neg($crate::_reg_idx!($idx1), $crate::_reg_idx8!($idx2)))
        }
    };
    { $($tt:tt)+ } => {
        Instr::ExtensionCodes(isa_instr! { $( $tt )+ })
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_block_ar {
    (a8) => {
        RegBlockAR::A
    };
    (a16) => {
        RegBlockAR::A
    };
    (a32) => {
        RegBlockAR::A
    };
    (a64) => {
        RegBlockAR::A
    };
    (a128) => {
        RegBlockAR::A
    };
    (a256) => {
        RegBlockAR::A
    };
    (a512) => {
        RegBlockAR::A
    };
    (a1024) => {
        RegBlockAFR::A
    };

    (r128) => {
        RegBlockAR::R
    };
    (r160) => {
        RegBlockAR::R
    };
    (r256) => {
        RegBlockAR::R
    };
    (r512) => {
        RegBlockAR::R
    };
    (r1024) => {
        RegBlockAR::R
    };
    (r2048) => {
        RegBlockAR::R
    };
    (r4096) => {
        RegBlockAR::R
    };
    (r8192) => {
        RegBlockAR::R
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_block {
    (a8) => {
        RegBlockAFR::A
    };
    (a16) => {
        RegBlockAFR::A
    };
    (a32) => {
        RegBlockAFR::A
    };
    (a64) => {
        RegBlockAFR::A
    };
    (a128) => {
        RegBlockAFR::A
    };
    (a256) => {
        RegBlockAFR::A
    };
    (a512) => {
        RegBlockAFR::A
    };
    (a1024) => {
        RegBlockAFR::A
    };

    (f16b) => {
        RegBlockAFR::F
    };
    (f16) => {
        RegBlockAFR::F
    };
    (f32) => {
        RegBlockAFR::F
    };
    (f64) => {
        RegBlockAFR::F
    };
    (f80) => {
        RegBlockAFR::F
    };
    (f128) => {
        RegBlockAFR::F
    };
    (f256) => {
        RegBlockAFR::F
    };
    (f512) => {
        RegBlockAFR::F
    };

    (r128) => {
        RegBlockAFR::R
    };
    (r160) => {
        RegBlockAFR::R
    };
    (r256) => {
        RegBlockAFR::R
    };
    (r512) => {
        RegBlockAFR::R
    };
    (r1024) => {
        RegBlockAFR::R
    };
    (r2048) => {
        RegBlockAFR::R
    };
    (r4096) => {
        RegBlockAFR::R
    };
    (r8192) => {
        RegBlockAFR::R
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_sfx {
    ($a:ident, $b:ident,a8) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a16) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a32) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a64) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a128) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a256) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a512) => {
        $crate::paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a1024) => {
        $crate::paste! { $a :: [<$b A>] }
    };

    ($a:ident, $b:ident,f16b) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f16) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f32) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f64) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f80) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f128) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f256) => {
        $crate::paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f512) => {
        $crate::paste! { $a :: [<$b F>] }
    };

    ($a:ident, $b:ident,r128) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r160) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r256) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r512) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r1024) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r2048) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r4096) => {
        $crate::paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r8192) => {
        $crate::paste! { $a :: [<$b R>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_ty {
    ($ident:ident,a8) => {
        $crate::paste! { [<$ident A>] :: A8 }
    };
    ($ident:ident,a16) => {
        $crate::paste! { [<$ident A>] :: A16 }
    };
    ($ident:ident,a32) => {
        $crate::paste! { [<$ident A>] :: A32 }
    };
    ($ident:ident,a64) => {
        $crate::paste! { [<$ident A>] :: A64 }
    };
    ($ident:ident,a128) => {
        $crate::paste! { [<$ident A>] :: A128 }
    };
    ($ident:ident,a256) => {
        $crate::paste! { [<$ident A>] :: A256 }
    };
    ($ident:ident,a512) => {
        $crate::paste! { [<$ident A>] :: A512 }
    };
    ($ident:ident,a1024) => {
        $crate::paste! { [<$ident A>] :: A1024 }
    };

    ($ident:ident,f16b) => {
        $crate::paste! { [<$ident F>] :: F16B }
    };
    ($ident:ident,f16) => {
        $crate::paste! { [<$ident F>] :: F16 }
    };
    ($ident:ident,f32) => {
        $crate::paste! { [<$ident F>] :: F32 }
    };
    ($ident:ident,f64) => {
        $crate::paste! { [<$ident F>] :: F64 }
    };
    ($ident:ident,f80) => {
        $crate::paste! { [<$ident F>] :: F80 }
    };
    ($ident:ident,f128) => {
        $crate::paste! { [<$ident F>] :: F128 }
    };
    ($ident:ident,f256) => {
        $crate::paste! { [<$ident F>] :: F256 }
    };
    ($ident:ident,f512) => {
        $crate::paste! { [<$ident F>] :: F512 }
    };

    ($ident:ident,r128) => {
        $crate::paste! { [<$ident R>] :: R128 }
    };
    ($ident:ident,r160) => {
        $crate::paste! { [<$ident R>] :: R160 }
    };
    ($ident:ident,r256) => {
        $crate::paste! { [<$ident R>] :: R256 }
    };
    ($ident:ident,r512) => {
        $crate::paste! { [<$ident R>] :: R512 }
    };
    ($ident:ident,r1024) => {
        $crate::paste! { [<$ident R>] :: R1024 }
    };
    ($ident:ident,r2048) => {
        $crate::paste! { [<$ident R>] :: R2048 }
    };
    ($ident:ident,r4096) => {
        $crate::paste! { [<$ident R>] :: R4096 }
    };
    ($ident:ident,r8192) => {
        $crate::paste! { [<$ident R>] :: R8192 }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tya2 {
    ($ident:ident,a8) => {
        $crate::paste! { [<$ident A2>] :: A8 }
    };
    ($ident:ident,a16) => {
        $crate::paste! { [<$ident A2>] :: A16 }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tya {
    ($ident:ident,a8) => {
        $crate::paste! { [<$ident A>] :: A8 }
    };
    ($ident:ident,a16) => {
        $crate::paste! { [<$ident A>] :: A16 }
    };
    ($ident:ident,a32) => {
        $crate::paste! { [<$ident A>] :: A32 }
    };
    ($ident:ident,a64) => {
        $crate::paste! { [<$ident A>] :: A64 }
    };
    ($ident:ident,a128) => {
        $crate::paste! { [<$ident A>] :: A128 }
    };
    ($ident:ident,a256) => {
        $crate::paste! { [<$ident A>] :: A256 }
    };
    ($ident:ident,a512) => {
        $crate::paste! { [<$ident A>] :: A512 }
    };
    ($ident:ident,a1024) => {
        $crate::paste! { [<$ident A>] :: A1024 }
    };
    ($ident:ident, $other:ident) => {
        panic!("operation requires `A` register")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tyf {
    ($ident:ident,f16b) => {
        $crate::paste! { [<$ident F>] :: F16B }
    };
    ($ident:ident,f16) => {
        $crate::paste! { [<$ident F>] :: F16 }
    };
    ($ident:ident,f32) => {
        $crate::paste! { [<$ident F>] :: F32 }
    };
    ($ident:ident,f64) => {
        $crate::paste! { [<$ident F>] :: F64 }
    };
    ($ident:ident,f80) => {
        $crate::paste! { [<$ident F>] :: F80 }
    };
    ($ident:ident,f128) => {
        $crate::paste! { [<$ident F>] :: F128 }
    };
    ($ident:ident,f256) => {
        $crate::paste! { [<$ident F>] :: F256 }
    };
    ($ident:ident,f512) => {
        $crate::paste! { [<$ident F>] :: F512 }
    };
    ($ident:ident, $other:ident) => {
        panic!("operation requires `F` register")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tyr {
    ($ident:ident,r128) => {
        $crate::paste! { [<$ident R>] :: R128 }
    };
    ($ident:ident,r160) => {
        $crate::paste! { [<$ident R>] :: R160 }
    };
    ($ident:ident,r256) => {
        $crate::paste! { [<$ident R>] :: R256 }
    };
    ($ident:ident,r512) => {
        $crate::paste! { [<$ident R>] :: R512 }
    };
    ($ident:ident,r1024) => {
        $crate::paste! { [<$ident R>] :: R1024 }
    };
    ($ident:ident,r2048) => {
        $crate::paste! { [<$ident R>] :: R2048 }
    };
    ($ident:ident,r4096) => {
        $crate::paste! { [<$ident R>] :: R4096 }
    };
    ($ident:ident,r8192) => {
        $crate::paste! { [<$ident R>] :: R8192 }
    };
    ($ident:ident, $other:ident) => {
        panic!("operation requires `R` register")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tyar {
    (a8) => {
        RegAR::A(RegA::A8)
    };
    (a16) => {
        RegAR::A(RegA::A16)
    };
    (a32) => {
        RegAR::A(RegA::A32)
    };
    (a64) => {
        RegAR::A(RegA::A64)
    };
    (a128) => {
        RegAR::A(RegA::A128)
    };
    (a256) => {
        RegAR::A(RegA::A256)
    };
    (a512) => {
        RegAR::A(RegA::A512)
    };
    (a1024) => {
        RegAR::A(RegA::A1024)
    };

    (r128) => {
        RegAR::R(RegR::R128)
    };
    (r160) => {
        RegAR::R(RegR::R160)
    };
    (r256) => {
        RegAR::R(RegR::R256)
    };
    (r512) => {
        RegAR::R(RegR::R512)
    };
    (r1024) => {
        RegAR::R(RegR::R1024)
    };
    (r2048) => {
        RegAR::R(RegR::R2048)
    };
    (r4096) => {
        RegAR::R(RegR::R4096)
    };
    (r8192) => {
        RegAR::R(RegR::R8192)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_idx {
    ($idx:literal) => {
        $crate::paste! { Reg32::[<Reg $idx>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_idx8 {
    ($idx:literal) => {
        $crate::paste! { Reg8::[<Reg $idx>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_idx16 {
    ($idx:literal) => {
        $crate::paste! { Reg16::[<Reg $idx>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _merge_flag {
    (s) => {
        MergeFlag::Set
    };
    (a) => {
        MergeFlag::Add
    };
    (n) => {
        MergeFlag::And
    };
    (o) => {
        MergeFlag::Or
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _rounding_flag {
    (z) => {
        RoundingFlag::TowardsZero
    };
    (n) => {
        RoundingFlag::TowardsNearest
    };
    (f) => {
        RoundingFlag::Floor
    };
    (c) => {
        RoundingFlag::Ceil
    };
    ($other:ident) => {
        panic!("wrong float rounding flag")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _int_flags {
    (uc) => {
        IntFlags::unsigned_checked()
    };
    (cu) => {
        IntFlags::unsigned_checked()
    };
    (sc) => {
        IntFlags::signed_checked()
    };
    (cs) => {
        IntFlags::signed_checked()
    };
    (uw) => {
        IntFlags::unsigned_wrapped()
    };
    (wu) => {
        IntFlags::unsigned_wrapped()
    };
    (sw) => {
        IntFlags::signed_wrapped()
    };
    (ws) => {
        IntFlags::signed_wrapped()
    };
    ($other:ident) => {
        panic!("wrong integer operation flags")
    };
}
