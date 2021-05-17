// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[cfg(feature = "std")]
#[macro_export]
macro_rules! aluasm {
    ($( $tt:tt )+) => { {
        let mut code: Vec<::alure::Instr<::alure::instr::Nop>> = vec![];
        aluasm_inner! { code => $( $tt )+ };
        ::alure::Lib(code)
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

    (zero $reg:ident [ $idx:literal ]) => {
        Instr::Put(_reg_sfx!(PutOp, Zero, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
        ))
    };

    (cl $reg:ident [ $idx:literal ]) => {
        Instr::Put(_reg_sfx!(PutOp, Cl, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
        ))
    };

    (put $reg:ident [ $idx:literal ], $val:tt) => {
        Instr::Put(_reg_sfx!(PutOp, Put, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            Value::from_str(stringify!($val)).expect("invalid hex literal"),
        ))
    };

    (putif $reg:ident [ $idx:literal ], $val:tt) => {
        Instr::Put(_reg_sfx!(PutOp, PutIf, $reg)(
            _reg_ty!(Reg, $reg),
            _reg_idx!($idx),
            Value::from_str(stringify!($val)).expect("invalid hex literal"),
        ))
    };

    (swp $reg1:ident [ $idx1:literal ] , $reg2:ident [ $idx2:literal ]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlock::A, RegBlock::A) => Instr::Move(MoveOp::SwpA(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_a().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_a().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlock::R, RegBlock::R) => Instr::Move(MoveOp::SwpR(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_r().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_r().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlock::A, RegBlock::R) => Instr::Move(MoveOp::SwpAR(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_a().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_r().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlock::R, RegBlock::A) => panic!(
                "Wrong order of registers in `swp` operation. Use A-register first"
            ),
        }
    };

    (mov $reg1:ident [ $idx1:literal ] , $reg2:ident [ $idx2:literal ]) => {
        match (_reg_block!($reg1), _reg_block!($reg2)) {
            (RegBlock::A, RegBlock::A) => Instr::Move(MoveOp::MovA(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_a().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_a().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlock::R, RegBlock::R) => Instr::Move(MoveOp::MovR(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_r().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_r().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlock::A, RegBlock::R) => Instr::Move(MoveOp::MovAR(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_a().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_r().unwrap(),
                _reg_idx!($idx2),
            )),
            (RegBlock::R, RegBlock::A) => Instr::Move(MoveOp::MovRA(
                Reg::from(_reg_ty!(Reg, $reg1)).reg_r().unwrap(),
                _reg_idx!($idx1),
                Reg::from(_reg_ty!(Reg, $reg2)).reg_a().unwrap(),
                _reg_idx!($idx2),
            )),
        }
    };

    (amov:u $reg1:ident , $reg2:ident) => {
        Instr::Move(MoveOp::AMov(_reg_ty!(Reg, $reg1), _reg_ty!(Reg, $reg2), NumType::Unsigned))
    };
    (amov:s $reg1:ident , $reg2:ident) => {
        Instr::Move(MoveOp::AMov(_reg_ty!(Reg, $reg1), _reg_ty!(Reg, $reg2), NumType::Signed))
    };
    (amov:f $reg1:ident , $reg2:ident) => {
        Instr::Move(MoveOp::AMov(_reg_ty!(Reg, $reg1), _reg_ty!(Reg, $reg2), NumType::Float23))
    };
    (amov:d $reg1:ident , $reg2:ident) => {
        Instr::Move(MoveOp::AMov(_reg_ty!(Reg, $reg1), _reg_ty!(Reg, $reg2), NumType::Float52))
    };
}

#[doc(hidden)]
#[cfg(feature = "std")]
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
    { $code:ident => $op:ident $arg:ident [ $idx:literal ] <- $arglit:tt ; $($tt:tt)* } => {
        $code.push(instr!{ $op $arg [ $idx ], $arglit });
        aluasm_inner! { $code => $( $tt )* }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_block {
    (ap) => {
        RegBlock::A
    };
    (a8) => {
        RegBlock::A
    };
    (a16) => {
        RegBlock::A
    };
    (a32) => {
        RegBlock::A
    };
    (a64) => {
        RegBlock::A
    };
    (a128) => {
        RegBlock::A
    };
    (a256) => {
        RegBlock::A
    };
    (a512) => {
        RegBlock::A
    };

    (r128) => {
        RegBlock::R
    };
    (r160) => {
        RegBlock::R
    };
    (r256) => {
        RegBlock::R
    };
    (r512) => {
        RegBlock::R
    };
    (r1024) => {
        RegBlock::R
    };
    (r2048) => {
        RegBlock::R
    };
    (r4096) => {
        RegBlock::R
    };
    (r8192) => {
        RegBlock::R
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_sfx {
    ($a:ident, $b:ident, ap) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a8) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a16) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a32) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a64) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a128) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a256) => {
        paste! { $a :: [<$b A>] }
    };
    ($a:ident, $b:ident, a512) => {
        paste! { $a :: [<$b A>] }
    };

    ($a:ident, $b:ident, r128) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r160) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r256) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r512) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r1024) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r2048) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r4096) => {
        paste! { $a :: [<$b R>] }
    };
    ($a:ident, $b:ident, r8192) => {
        paste! { $a :: [<$b R>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _reg_ty {
    ($ident:ident, ap) => {
        paste! { [<$ident A>] :: AP }
    };
    ($ident:ident, a8) => {
        paste! { [<$ident A>] :: A8 }
    };
    ($ident:ident, a16) => {
        paste! { [<$ident A>] :: A16 }
    };
    ($ident:ident, a32) => {
        paste! { [<$ident A>] :: A32 }
    };
    ($ident:ident, a64) => {
        paste! { [<$ident A>] :: A64 }
    };
    ($ident:ident, a128) => {
        paste! { [<$ident A>] :: A128 }
    };
    ($ident:ident, a256) => {
        paste! { [<$ident A>] :: A256 }
    };
    ($ident:ident, a512) => {
        paste! { [<$ident A>] :: A512 }
    };

    ($ident:ident, r128) => {
        paste! { [<$ident R>] :: R128 }
    };
    ($ident:ident, r160) => {
        paste! { [<$ident R>] :: R160 }
    };
    ($ident:ident, r256) => {
        paste! { [<$ident R>] :: R256 }
    };
    ($ident:ident, r512) => {
        paste! { [<$ident R>] :: R512 }
    };
    ($ident:ident, r1024) => {
        paste! { [<$ident R>] :: R1024 }
    };
    ($ident:ident, r2048) => {
        paste! { [<$ident R>] :: R2048 }
    };
    ($ident:ident, r4096) => {
        paste! { [<$ident R>] :: R4096 }
    };
    ($ident:ident, r8192) => {
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
