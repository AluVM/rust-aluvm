// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

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
    { $code:ident => $op:ident $( $arg:ident [ $idx:literal ] ),+ ; $($tt:tt)* } => {
        $code.push(instr!{ $op $( $arg [ $idx ]  ),+ });
        aluasm_inner! { $code => $( $tt )* }
    }
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! aluasm {
    ($( $tt:tt )+) => { {
        let mut code: Vec<::alure::Instr<::alure::instr::Nop>> = vec![];
        aluasm_inner! { code => $( $tt )+ };
        ::alure::Lib(code)
    } }
}

/*
macro_rules! args {
    ($arg:expr) => {};

    ($arg:expr , $($tt:tt)*) => {};

    ($reg:ident [ $idx:literal ]) => {};

    ($reg:ident [ $idx:literal ] , $($tt:tt)*) => {};
}
 */

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
        Instr::Put(reg_suffix!(PutOp, Zero, $reg)(
            reg_ext!(Reg, $reg),
            reg_idx!($idx),
        ))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! reg_suffix {
    ($a:ident, $b:ident, a8) => {
        paste! { $a :: [<$b A>] }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! reg_ext {
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
macro_rules! reg_idx {
    ($idx:literal) => {
        paste! { Reg32::[<Reg $idx>] }
    };
}
