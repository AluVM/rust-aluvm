// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![feature(trace_macros)]
#![feature(log_syntax)]

#[macro_use]
extern crate alure;

#[macro_use]
extern crate paste;

use alure::instr::{
    ArithmeticOp, CmpOp, ControlFlowOp, Instr, MoveOp, Nop, NumType, PutOp,
};
use alure::registers::{Reg, Reg32, RegA, RegBlock, RegR};
use alure::Value;
use std::str::FromStr;

trace_macros!(true);

fn main() {
    let code = aluasm! {
        zero    a8[1]                           ;
        cl      r1024[5]                        ;
        put     a16[8] <- 378                   ;
        putif   r1024[5] <- 0xaf67937b5498dc    ;
        swp     a8[1], a16[2]                   ;
        swp     r256[8], r256[7]                ;
        swp     a256[1], r256[7]                ;
        mov     a8[1], a16[2]                   ;
        mov     r256[8], r256[7]                ;
        mov     a256[1], r256[7]                ;
        mov     r512[4], a512[3]                ;
        amov:u  a256,a128                       ;
        amov:s  a256,a128                       ;
        amov:f  a256,a128                       ;
        amov:d  a256,a128                       ;
        ret                                     ;
        jmp     0                               ;
    };

    println!("\n\nentry:\n{}\n", code);
}
