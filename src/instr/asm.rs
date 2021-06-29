// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://github.com/internet2-org/aluvm-spec>
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

/// Macro compiler for AluVM assembler.
///
/// # Example
///
/// ```
/// # extern crate alloc;
/// # use paste::paste;
/// # use aluvm::*;
/// # use aluvm::instr::NOp;
///
/// let code = aluasm! {
///         clr     r1024[5]                        ;
///         put     378,a16[8]                      ;
///         putif   0xaf67937b5498dc,r128[5]        ;
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
/// let lib = Lib::<NOp>::with(code, None).unwrap();
/// let mut runtime = Vm::with(lib);
/// match runtime.main() {
///     Ok(true) => println!("success"),
///     Ok(false) => println!("execution reported validation failure"),
///     Err(err) => eprintln!("{}", err),
/// }
/// ```
#[macro_export]
macro_rules! aluasm {
    ($( $tt:tt )+) => { {
        use core::str::FromStr;
        use alloc::boxed::Box;

        use aluvm::instr::{
            ArithmeticOp, BitwiseOp, CmpOp, ControlFlowOp, DigestOp, FloatEqFlag, Instr, IntFlags,
            MergeFlag, MoveOp, NOp, PutOp, RoundingFlag, Secp256k1Op, SignFlag,
        };
        use aluvm::{
            Reg16, Reg32, Reg8, RegA, RegA2, RegBlockAFR, RegBlockAR, RegF, RegR, number,
        };

        let mut code: Vec<Instr<NOp>> = vec![];
        #[allow(unreachable_code)] {
            aluasm_inner! { code => $( $tt )+ };
        }
        code
    } }
}

#[doc(hidden)]
#[macro_export]
macro_rules! aluasm_inner {
    { $code:ident => } => { };
    { $code:ident => $op:ident ; $($tt:tt)* } => {
        $code.push(instr!{ $op });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:literal ),+ ; $($tt:tt)* } => {
        $code.push(instr!{ $op $( $arg ),+ });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:ident ),+ ; $($tt:tt)* } => {
        $code.push(instr!{ $op $( $arg ),+ });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident : $flag:ident $( $arg:ident ),+ ; $($tt:tt)* } => {
        $code.push(instr!{ $op : $flag $( $arg ),+ });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push(instr!{ $op $( $arg [ $idx ]  ),+ });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:ident [ $idx:literal ] ),+ .none=true ; $($tt:tt)* } => {
        $code.push(instr!{ $op $( $arg [ $idx ]  ),+ , true });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $( $arg:ident [ $idx:literal ] ),+ .none=false ; $($tt:tt)* } => {
        $code.push(instr!{ $op $( $arg [ $idx ]  ),+ , false });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident : $flag:ident $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push(instr!{ $op : $flag $( $arg [ $idx ]  ),+ });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident $arglit:literal , $arg:ident [ $idx:literal ] ; $($tt:tt)* } => {
        $code.push(instr!{ $op $arglit , $arg [ $idx ] });
        aluasm_inner! { $code => $( $tt )* }
    };
    { $code:ident => $op:ident : $flag:ident $arg:ident [ $idx:literal ], $arglit:expr ; $($tt:tt)* } => {
        $code.push(instr!{ $op : $flag $arg [ $idx ], $arglit });
        aluasm_inner! { $code => $( $tt )* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! instr {
    (fail) => {
        Instr::ControlFlow(ControlFlowOp::Fail)
    };
    (succ) => {
        Instr::ControlFlow(ControlFlowOp::Succ)
    };
    (jmp $offset:literal) => {
        Instr::ControlFlow(ControlFlowOp::Jmp($offset))
    };
    (jif $offset:literal) => {
        Instr::ControlFlow(ControlFlowOp::Jif($offset))
    };
    (routine $offset:literal) => {
        Instr::ControlFlow(ControlFlowOp::Reutine($offset))
    };
    (call $offset:literal @ $lib:literal) => {
        Instr::ControlFlow(ControlFlowOp::Call(LibSite::with($offset, $lib)))
    };
    (exec $offset:literal @ $lib:literal) => {
        Instr::ControlFlow(ControlFlowOp::Exec(LibSite::with($offset, $lib)))
    };
    (ret) => {
        Instr::ControlFlow(ControlFlowOp::Ret)
    };

    (clr $reg:ident[$idx:literal]) => {
        Instr::Put(_reg_sfx!(PutOp, Clr, $reg)(_reg_ty!(Reg, $reg), _reg_idx!($idx)))
    };

    (put $val:literal, $reg:ident[$idx:literal]) => {
        Instr::Put(_reg_sfx!(PutOp, Put, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            Box::new(FromStr::from_str(stringify!($val)).expect("invalid hex literal")),
        ))
    };

    (putif $val:literal, $reg:ident[$idx:literal]) => {
        Instr::Put(_reg_sfx!(PutOp, PutIf, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            Box::new(FromStr::from_str(stringify!($val)).expect("invalid hex literal")),
        ))
    };

    (swp $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) {
            panic!("Swap operation must be performed between registers of the same type");
        }
        Instr::Move(_reg_sfx!(MoveOp, Swp, $reg1)(
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (mov $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) {
            panic!("Move operation must be performed between registers of the same type");
        }
        Instr::Move(_reg_sfx!(MoveOp, Mov, $reg1)(
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (dup $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) {
            panic!("Dup operation must be performed between registers of the same type");
        }
        Instr::Move(_reg_sfx!(MoveOp, Dup, $reg1)(
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (cpy $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) {
            panic!("Copy operation must be performed between registers of the same type");
        }
        Instr::Move(_reg_sfx!(MoveOp, Cpy, $reg1)(
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_ty!(Reg, $reg2),
            _reg_idx!($idx2),
        ))
    }};
    (cnv $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAFR::A, RegBlockAFR::F) => Instr::Move(MoveOp::CnvAF(
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_tyf!(Reg, $reg2),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::F, RegBlockAFR::A) => Instr::Move(MoveOp::CnvFA(
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_tya!(Reg, $reg2),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Move(MoveOp::CnvA(
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Move(MoveOp::CnvF(
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx2),
            )),
            (_, _) => panic!("Conversion operation between unsupported register types"),
        }
    }};
    (spy $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAFR::A, RegBlockAFR::R) => Instr::Move(MoveOp::SpyAR(
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_tyr!(Reg, $reg2),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::R, RegBlockAFR::A) => Instr::Move(MoveOp::SpyAR(
                _reg_tya!(Reg, $reg2),
                _reg_idx!($idx2),
                _reg_tyr!(Reg, $reg1),
                _reg_idx!($idx1),
            )),
            (_, _) => {
                panic!("Swap-conversion operation is supported only between A and R registers")
            }
        }
    }};

    (gt $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        if _reg_block!($reg1) != RegBlockAFR::R {
            panic!("`gt` operation for arithmetic registers requires suffix");
        }
        Instr::Cmp(CmpOp::GtR(_reg_tyr!(Reg, $reg1), _reg_idx!($idx1), _reg_idx!($idx2)))
    }};
    (gt: u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            SignFlag::Unsigned,
            _reg_tya!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (gt: s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            SignFlag::Signed,
            _reg_tya!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (gt: e $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtF(
            FloatEqFlag::Exact,
            _reg_tyf!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (gt: r $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtF(
            FloatEqFlag::Rounding,
            _reg_tyf!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        if _reg_block!($reg1) != RegBlockAFR::R {
            panic!("`lt` operation for arithmetic registers requires suffix");
        }
        Instr::Cmp(CmpOp::LtR(_reg_tyr!(Reg, $reg1), _reg_idx!($idx1), _reg_idx!($idx2)))
    }};
    (lt: u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            SignFlag::Unsigned,
            _reg_tya!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt: s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            SignFlag::Signed,
            _reg_tya!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt: e $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtF(
            FloatEqFlag::Exact,
            _reg_tyf!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt: r $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtF(
            FloatEqFlag::Rounding,
            _reg_tyf!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (eq $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal], $bool:literal) => {{
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) {
            panic!(
                "Equivalence check must be performed between registers of the same type and size"
            );
        }
        match _reg_block!($reg1) {
            RegBlockAFR::A => Instr::Cmp(CmpOp::EqA(
                $bool,
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            RegBlockAFR::R => Instr::Cmp(CmpOp::EqR(
                $bool,
                _reg_tyr!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            _ => panic!("Wrong registers for `eq` operation"),
        }
    }};
    (eq: e $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`eq` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::EqF(
            FloatEqFlag::Exact,
            _reg_tyf!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (eq: r $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`eq` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::EqF(
            FloatEqFlag::Rounding,
            _reg_tyf!(Reg, $reg1),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (ifn $reg:ident[$idx:literal]) => {
        match _reg_block!($reg) {
            RegBlockAFR::A => Instr::Cmp(CmpOp::IfNA(_reg_tya!(Reg, $reg), _reg_idx!($idx))),
            RegBlockAFR::R => Instr::Cmp(CmpOp::IfNR(_reg_tyr!(Reg, $reg), _reg_idx!($idx))),
            _ => panic!("Wrong registers for `ifn` operation"),
        }
    };
    (ifz $reg:ident[$idx:literal]) => {
        match _reg_block!($reg) {
            RegBlockAFR::A => Instr::Cmp(CmpOp::IfZA(_reg_tya!(Reg, $reg), _reg_idx!($idx))),
            RegBlockAFR::R => Instr::Cmp(CmpOp::IfZR(_reg_tyr!(Reg, $reg), _reg_idx!($idx))),
            _ => panic!("Wrong registers for `ifz` operation"),
        }
    };
    (st: $flag:ident $reg:ident[$idx:literal]) => {
        Instr::Cmp(CmpOp::St(_merge_flag!($flag), _reg_tya!(Reg, $reg), _reg_idx8!($idx)))
    };
    (inv st0) => {
        Instr::Cmp(CmpOp::StInv)
    };

    (add: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::AddA(
                _int_flags!($flag),
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::AddF(
                _rounding_flag!($flag),
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (a, b) if a == b => panic!("addition requires integer or float registers"),
            (_, _) if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) => {
                panic!("addition must be performed between registers of the same size")
            }
            (_, _) => panic!("addition must be performed between registers of the same type"),
        }
    };
    (sub: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::SubA(
                _int_flags!($flag),
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::SubF(
                _rounding_flag!($flag),
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (a, b) if a == b => panic!("subtraction requires integer or float registers"),
            (_, _) if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) => {
                panic!("subtraction must be performed between registers of the same size")
            }
            (_, _) => panic!("subtraction must be performed between registers of the same type"),
        }
    };
    (mul: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::MulA(
                _int_flags!($flag),
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::MulF(
                _rounding_flag!($flag),
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (a, b) if a == b => panic!("multiplication requires integer or float registers"),
            (_, _) if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) => {
                panic!("multiplication must be performed between registers of the same size")
            }
            (_, _) => panic!("multiplication must be performed between registers of the same type"),
        }
    };
    (div: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAFR::A, RegBlockAFR::A) => Instr::Arithmetic(ArithmeticOp::DivA(
                _int_flags!($flag),
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (RegBlockAFR::F, RegBlockAFR::F) => Instr::Arithmetic(ArithmeticOp::DivF(
                _rounding_flag!($flag),
                _reg_tyf!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            )),
            (a, b) if a == b => panic!("division requires integer or float registers"),
            (_, _) if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2) => {
                panic!("division must be performed between registers of the same size")
            }
            (_, _) => panic!("division must be performed between registers of the same type"),
        }
    };
    (rem $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::A || _reg_block!($reg2) != RegBlockAFR::A {
            panic!("modulo division must be performed only using integer arithmetic registers");
        } else {
            Instr::Arithmetic(ArithmeticOp::Rem(
                _reg_tya!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_tya!(Reg, $reg2),
                _reg_idx!($idx2),
            ))
        }
    };
    (inc $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            _reg_tya!(Reg, $reg),
            _reg_idx!($idx),
            number::Step::with(1),
        ))
    };
    (add $step:literal, $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            _reg_tya!(Reg, $reg),
            _reg_idx!($idx),
            number::Step::with($step),
        ))
    };
    (dec $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            _reg_tya!(Reg, $reg),
            _reg_idx!($idx),
            number::Step::with(-1),
        ))
    };
    (sub $step:literal, $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            _reg_tya!(Reg, $reg),
            _reg_idx!($idx),
            number::Step::with($step * -1),
        ))
    };
    (neg $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Neg(_reg_ty!(Reg, $reg).into(), _reg_idx16!($idx)))
    };
    (abs $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Abs(_reg_ty!(Reg, $reg).into(), _reg_idx16!($idx)))
    };

    (and $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal], $reg3:ident[$idx3:literal]) => {
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2)
            || _reg_ty!(Reg, $reg2) != _reg_ty!(Reg, $reg3)
        {
            panic!("`and` operation must use the same type of registers for all of its operands");
        } else if _reg_block!($reg1) != RegBlockAFR::A && _reg_block!($reg1) != RegBlockAFR::R {
            panic!("`and` operation requires integer arithmetic or generic registers");
        } else {
            Instr::Bitwise(BitwiseOp::And(
                _reg_ty!(Reg, $reg1).into(),
                _reg_idx16!($idx1),
                _reg_idx16!($idx2),
                _reg_idx16!($idx3),
            ))
        }
    };
    (or $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal], $reg3:ident[$idx3:literal]) => {
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2)
            || _reg_ty!(Reg, $reg2) != _reg_ty!(Reg, $reg3)
        {
            panic!("`or` operation must use the same type of registers for all of its operands");
        } else if _reg_block!($reg1) != RegBlockAFR::A && _reg_block!($reg1) != RegBlockAFR::R {
            panic!("`or` operation requires integer arithmetic or generic registers");
        } else {
            Instr::Bitwise(BitwiseOp::Or(
                _reg_ty!(Reg, $reg1).into(),
                _reg_idx16!($idx1),
                _reg_idx16!($idx2),
                _reg_idx16!($idx3),
            ))
        }
    };
    (xor $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal], $reg3:ident[$idx3:literal]) => {
        if _reg_ty!(Reg, $reg1) != _reg_ty!(Reg, $reg2)
            || _reg_ty!(Reg, $reg2) != _reg_ty!(Reg, $reg3)
        {
            panic!("`xor` operation must use the same type of registers for all of its operands");
        } else if _reg_block!($reg1) != RegBlockAFR::A && _reg_block!($reg1) != RegBlockAFR::R {
            panic!("`xor` operation requires integer arithmetic or generic registers");
        } else {
            Instr::Bitwise(BitwiseOp::Xor(
                _reg_ty!(Reg, $reg1).into(),
                _reg_idx16!($idx1),
                _reg_idx16!($idx2),
                _reg_idx16!($idx3),
            ))
        }
    };
    (shl $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::Shl(
            _reg_tya2!(Reg, $reg2),
            _reg_idx!($idx2),
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
        ))
    };
    (shr: u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::ShrA(
            SignFlag::Unsigned,
            _reg_tya2!(Reg, $reg2),
            _reg_idx16!($idx2),
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
        ))
    };
    (shr: s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::ShrA(
            SignFlag::Signed,
            _reg_tya2!(Reg, $reg2),
            _reg_idx16!($idx2),
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
        ))
    };
    (shr $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::ShrR(
            _reg_tya2!(Reg, $reg2),
            _reg_idx!($idx2),
            _reg_ty!(Reg, $reg1),
            _reg_idx!($idx1),
        ))
    };
    (scl $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::Scl(
            _reg_tya2!(Reg, $reg2),
            _reg_idx!($idx2),
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
        ))
    };
    (scr $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        Instr::Bitwise(BitwiseOp::Scr(
            _reg_tya2!(Reg, $reg2),
            _reg_idx!($idx2),
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
        ))
    };
    (rev $reg:ident[$idx:literal]) => {
        match _reg_block!($reg) {
            RegBlockAFR::A => {
                Instr::Bitwise(BitwiseOp::RevA(_reg_tya!(Reg, $reg), _reg_idx!($idx)))
            }
            RegBlockAFR::R => {
                Instr::Bitwise(BitwiseOp::RevR(_reg_tyr!(Reg, $reg), _reg_idx!($idx)))
            }
            _ => panic!("Wrong registers for `rev` operation"),
        }
    };

    (ripemd s16[$idx1:literal],r160[$idx2:literal]) => {
        Instr::Digest(DigestOp::Ripemd(_reg_idx!($idx1), _reg_idx8!($idx2)))
    };
    (sha2 s16[$idx1:literal],r256[$idx2:literal]) => {
        Instr::Digest(DigestOp::Sha256(_reg_idx!($idx1), _reg_idx8!($idx2)))
    };
    (sha2 s16[$idx1:literal],r512[$idx2:literal]) => {
        Instr::Digest(DigestOp::Sha512(_reg_idx!($idx1), _reg_idx8!($idx2)))
    };

    (secpgen $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::R || _reg_block!($reg2) != RegBlockAFR::R {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Gen(_reg_idx!($idx1), _reg_idx8!($idx2)))
        }
    };
    (
        secpmul $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal], $reg3:ident[$idx3:literal]
    ) => {
        if _reg_ty!(Reg, $reg2) != _reg_ty!(Reg, $reg3) {
            panic!("ecmul instruction can be used only with registers of the same type");
        } else {
            Instr::Secp256k1(Secp256k1Op::Mul(
                _reg_block_ar!($reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
                _reg_idx!($idx3),
            ))
        }
    };
    (secpadd $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::R || _reg_block!($reg2) != RegBlockAFR::R {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Add(_reg_idx!($idx1), _reg_idx8!($idx2)))
        }
    };
    (secpneg $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::R || _reg_block!($reg2) != RegBlockAFR::R {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Neg(_reg_idx!($idx1), _reg_idx8!($idx2)))
        }
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
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a16) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a32) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a64) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a128) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a256) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a512) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident,a1024) => {
        paste! { $a :: [<$b A>] }
    };

    ($a:ident, $b:ident,f16b) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f16) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f32) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f64) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f80) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f128) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f256) => {
        paste! { $a :: [<$b F>] }
    };
    ($a:ident, $b:ident,f512) => {
        paste! { $a :: [<$b F>] }
    };

    ($a:ident, $b:ident,r128) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r160) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r256) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r512) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r1024) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r2048) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r4096) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident,r8192) => {
        paste! { $a :: [<$b R>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_ty {
    ($ident:ident,a8) => {
        paste! { [<$ident A>] :: A8 }
    };
    ($ident:ident,a16) => {
        paste! { [<$ident A>] :: A16 }
    };
    ($ident:ident,a32) => {
        paste! { [<$ident A>] :: A32 }
    };
    ($ident:ident,a64) => {
        paste! { [<$ident A>] :: A64 }
    };
    ($ident:ident,a128) => {
        paste! { [<$ident A>] :: A128 }
    };
    ($ident:ident,a256) => {
        paste! { [<$ident A>] :: A256 }
    };
    ($ident:ident,a512) => {
        paste! { [<$ident A>] :: A512 }
    };
    ($ident:ident,a1024) => {
        paste! { [<$ident A>] :: A1024 }
    };

    ($ident:ident,f16b) => {
        paste! { [<$ident F>] :: F16B }
    };
    ($ident:ident,f16) => {
        paste! { [<$ident F>] :: F16 }
    };
    ($ident:ident,f32) => {
        paste! { [<$ident F>] :: F32 }
    };
    ($ident:ident,f64) => {
        paste! { [<$ident F>] :: F64 }
    };
    ($ident:ident,f80) => {
        paste! { [<$ident F>] :: F80 }
    };
    ($ident:ident,f128) => {
        paste! { [<$ident F>] :: F128 }
    };
    ($ident:ident,f256) => {
        paste! { [<$ident F>] :: F256 }
    };
    ($ident:ident,f512) => {
        paste! { [<$ident F>] :: F512 }
    };

    ($ident:ident,r128) => {
        paste! { [<$ident R>] :: R128 }
    };
    ($ident:ident,r160) => {
        paste! { [<$ident R>] :: R160 }
    };
    ($ident:ident,r256) => {
        paste! { [<$ident R>] :: R256 }
    };
    ($ident:ident,r512) => {
        paste! { [<$ident R>] :: R512 }
    };
    ($ident:ident,r1024) => {
        paste! { [<$ident R>] :: R1024 }
    };
    ($ident:ident,r2048) => {
        paste! { [<$ident R>] :: R2048 }
    };
    ($ident:ident,r4096) => {
        paste! { [<$ident R>] :: R4096 }
    };
    ($ident:ident,r8192) => {
        paste! { [<$ident R>] :: R8192 }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tya2 {
    ($ident:ident,a8) => {
        paste! { [<$ident A2>] :: A8 }
    };
    ($ident:ident,a16) => {
        paste! { [<$ident A2>] :: A16 }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tya {
    ($ident:ident,a8) => {
        paste! { [<$ident A>] :: A8 }
    };
    ($ident:ident,a16) => {
        paste! { [<$ident A>] :: A16 }
    };
    ($ident:ident,a32) => {
        paste! { [<$ident A>] :: A32 }
    };
    ($ident:ident,a64) => {
        paste! { [<$ident A>] :: A64 }
    };
    ($ident:ident,a128) => {
        paste! { [<$ident A>] :: A128 }
    };
    ($ident:ident,a256) => {
        paste! { [<$ident A>] :: A256 }
    };
    ($ident:ident,a512) => {
        paste! { [<$ident A>] :: A512 }
    };
    ($ident:ident,a1024) => {
        paste! { [<$ident A>] :: A1024 }
    };
    ($ident:ident, $other:ident) => {
        panic!("operation requires `A` register")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tyf {
    ($ident:ident,f16b) => {
        paste! { [<$ident F>] :: F16B }
    };
    ($ident:ident,f16) => {
        paste! { [<$ident F>] :: F16 }
    };
    ($ident:ident,f32) => {
        paste! { [<$ident F>] :: F32 }
    };
    ($ident:ident,f64) => {
        paste! { [<$ident F>] :: F64 }
    };
    ($ident:ident,f80) => {
        paste! { [<$ident F>] :: F80 }
    };
    ($ident:ident,f128) => {
        paste! { [<$ident F>] :: F128 }
    };
    ($ident:ident,f256) => {
        paste! { [<$ident F>] :: F256 }
    };
    ($ident:ident,f512) => {
        paste! { [<$ident F>] :: F512 }
    };
    ($ident:ident, $other:ident) => {
        panic!("operation requires `F` register")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_tyr {
    ($ident:ident,r128) => {
        paste! { [<$ident R>] :: R128 }
    };
    ($ident:ident,r160) => {
        paste! { [<$ident R>] :: R160 }
    };
    ($ident:ident,r256) => {
        paste! { [<$ident R>] :: R256 }
    };
    ($ident:ident,r512) => {
        paste! { [<$ident R>] :: R512 }
    };
    ($ident:ident,r1024) => {
        paste! { [<$ident R>] :: R1024 }
    };
    ($ident:ident,r2048) => {
        paste! { [<$ident R>] :: R2048 }
    };
    ($ident:ident,r4096) => {
        paste! { [<$ident R>] :: R4096 }
    };
    ($ident:ident,r8192) => {
        paste! { [<$ident R>] :: R8192 }
    };
    ($ident:ident, $other:ident) => {
        panic!("operation requires `R` register")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_idx {
    ($idx:literal) => {
        paste! { Reg32::[<Reg $idx>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_idx8 {
    ($idx:literal) => {
        paste! { Reg8::[<Reg $idx>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_idx16 {
    ($idx:literal) => {
        paste! { Reg16::[<Reg $idx>] }
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
