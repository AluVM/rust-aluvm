// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use alure::instruction::{
    ArithmeticOp, CmpOp, ControlFlowOp, Instr, MoveOp, Nop,
};
use alure::registers::{Reg32, RegA};

macro_rules! aluasm {
    ($( $op:ident $($arg:expr),* ;)+) => {
        aluasm! { ::alure::instruction::Nop => $( $op $($arg),* ;)+ }
    };
    ($ext:ty => $( $op:ident $($arg:expr),* ;)+) => { {
        let mut code: Vec<Instr<$ext>> = vec![];
        $( code.push(instr!( $op $( $arg ),* )); )+
        code
    } };
}

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
}

fn main() {
    let code = aluasm! {
        ret;
        jmp 0;
    };

    println!("{:?}", code);
}
