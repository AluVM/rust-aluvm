// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_export]
macro_rules! aluasm {
    ($( $tt:tt )+) => { {
        let mut code: Vec<::alure::Instr<::alure::instr::NOp>> = vec![];
        aluasm_inner! { code => $( $tt )+ };
        code
    } }
}

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
            ::core::str::FromStr::from_str(stringify!($val)).expect("invalid hex literal"),
        ))
    };

    (putif $val:literal, $reg:ident[$idx:literal]) => {
        Instr::Put(_reg_sfx!(PutOp, PutIf, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            ::core::str::FromStr::from_str(stringify!($val)).expect("invalid hex literal"),
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

    /*
    (gt $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        if _reg_block!($reg1) != RegBlockAR::R {
            panic!(
                "`gt` operation for arithmetic registers requires prefix specifying used \
                 arithmetics"
            );
        }
        Instr::Cmp(CmpOp::GtR(
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx16!($idx1),
            _reg_ty!(Reg, $reg2).into(),
            _reg_idx!($idx2),
        ))
    }};
    (gt: u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            NumType::Unsigned,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (gt: s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            NumType::Signed,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (gt: f $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            NumType::Float23,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (gt: d $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::GtA(
            NumType::Float52,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`gt` operation may be applied only to the registers of the same family");
        }
        if _reg_block!($reg1) != RegBlockAR::R {
            panic!(
                "`gt` operation for arithmetic registers requires prefix specifying used \
                 arithmetics"
            );
        }
        Instr::Cmp(CmpOp::LtR(
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx16!($idx1),
            _reg_ty!(Reg, $reg2).into(),
            _reg_idx!($idx2),
        ))
    }};
    (lt: u $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            NumType::Unsigned,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt: s $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            NumType::Signed,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt: f $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            NumType::Float23,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (lt: d $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {{
        if _reg_block!($reg1) != _reg_block!($reg2) {
            panic!("`lt` operation may be applied only to the registers of the same family");
        }
        Instr::Cmp(CmpOp::LtA(
            NumType::Float52,
            _reg_ty!(Reg, $reg1).into(),
            _reg_idx!($idx1),
            _reg_idx!($idx2),
        ))
    }};
    (eq $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlockAR::A, RegBlockAR::A) => Instr::Cmp(CmpOp::EqA(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_a().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_a().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlockAR::R, RegBlockAR::R) => Instr::Cmp(CmpOp::EqR(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_r().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_r().unwrap(),
                _reg_idx!($idx2),
            )),
            _ => panic!("Wrong order of registers in `swp` operation. Use A-register first"),
        }
    };
    (len $reg:ident[$idx:literal]) => {
        Instr::Cmp(CmpOp::Len(_reg_ty!(Reg, $reg), _reg_idx!($idx)))
    };
    (cnt $reg:ident[$idx:literal]) => {
        Instr::Cmp(CmpOp::Cnt(_reg_ty!(Reg, $reg), _reg_idx!($idx)))
    };
    (st2a) => {
        Instr::Cmp(CmpOp::St2A)
    };
    (a2st) => {
        Instr::Cmp(CmpOp::A2St)
    };

    (neg $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Neg(_reg_ty!(Reg, $reg), _reg_idx!($idx)))
    };
    (inc: $flag:ident $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            IncDec::Inc,
            _arithmetic_flag!($flag),
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            u4::try_from(1).unwrap(),
        ))
    };
    (inc: $flag:ident $reg:ident[$idx:literal], $step:expr) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            IncDec::Inc,
            _arithmetic_flag!($flag),
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            u4::try_from($step).expect("scalar value for increment must be in 0..16 range"),
        ))
    };
    (dec: $flag:ident $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            IncDec::Dec,
            _arithmetic_flag!($flag),
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            u4::try_from(1).unwrap(),
        ))
    };
    (dec: $flag:ident $reg:ident[$idx:literal], $step:expr) => {
        Instr::Arithmetic(ArithmeticOp::Stp(
            IncDec::Dec,
            _arithmetic_flag!($flag),
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            u4::try_from($step).expect("scalar value for decrement must be in 0..16 range"),
        ))
    };
    (add: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAR::A || _reg_block!($reg2) != RegBlockAR::A {
            panic!("arithmetic instruction accept only arithmetic registers (A-registers)");
        } else {
            Instr::Arithmetic(ArithmeticOp::Add(
                _arithmetic_flag!($flag),
                _reg_ty!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            ))
        }
    };
    (sub: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAR::A || _reg_block!($reg2) != RegBlockAR::A {
            panic!("arithmetic instruction accept only arithmetic registers (A-registers)");
        } else {
            Instr::Arithmetic(ArithmeticOp::Sub(
                _arithmetic_flag!($flag),
                _reg_ty!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            ))
        }
    };
    (mul: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAR::A || _reg_block!($reg2) != RegBlockAR::A {
            panic!("arithmetic instruction accept only arithmetic registers (A-registers)");
        } else {
            Instr::Arithmetic(ArithmeticOp::Mul(
                _arithmetic_flag!($flag),
                _reg_ty!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            ))
        }
    };
    (div: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAR::A || _reg_block!($reg2) != RegBlockAR::A {
            panic!("arithmetic instruction accept only arithmetic registers (A-registers)");
        } else {
            Instr::Arithmetic(ArithmeticOp::Div(
                _arithmetic_flag!($flag),
                _reg_ty!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            ))
        }
    };
    (rem: $flag:ident $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAR::A || _reg_block!($reg2) != RegBlockAR::A {
            panic!("arithmetic instruction accept only arithmetic registers (A-registers)");
        } else {
            Instr::Arithmetic(ArithmeticOp::Div(
                _arithmetic_flag!($flag),
                _reg_ty!(Reg, $reg1),
                _reg_idx!($idx1),
                _reg_idx!($idx2),
            ))
        }
    };
    (abs $reg:ident[$idx:literal]) => {
        Instr::Arithmetic(ArithmeticOp::Abs(_reg_ty!(Reg, $reg), _reg_idx!($idx)))
    };
     */
    (ecgen: secp $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::R || _reg_block!($reg2) != RegBlockAFR::R {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Gen(_reg_idx!($idx1), _reg_idx8!($idx2)))
        }
    };
    (
        ecmul: secp
        $reg1:ident[$idx1:literal],
        $reg2:ident[$idx2:literal],
        $reg3:ident[$idx3:literal]
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
    (ecadd: secp $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::R || _reg_block!($reg2) != RegBlockAFR::R {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Add(_reg_idx!($idx1), _reg_idx8!($idx2)))
        }
    };
    (ecneg: secp $reg1:ident[$idx1:literal], $reg2:ident[$idx2:literal]) => {
        if _reg_block!($reg1) != RegBlockAFR::R || _reg_block!($reg2) != RegBlockAFR::R {
            panic!("elliptic curve instruction accept only generic registers (R-registers)");
        } else {
            Instr::Secp256k1(Secp256k1Op::Neg(_reg_idx!($idx1), _reg_idx8!($idx2)))
        }
    };
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
