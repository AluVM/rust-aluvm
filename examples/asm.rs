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

use alure::instr::bytecode::disassemble;
use alure::instr::{
    ArithmeticOp, Arithmetics, CmpOp, ControlFlowOp, IncDec, Instr, MoveOp, NOp, NumType, PutOp,
    Secp256k1Op,
};
use alure::{Lib, Runtime};
use alure::{Reg, Reg16, Reg32, Reg8, RegA, RegBlock, RegR};
use amplify_num::hex::ToHex;
use amplify_num::u4;
use std::convert::TryFrom;

trace_macros!(true);

fn main() {
    let code = aluasm! {
        zero    a8[1]                           ;
        cl      r1024[5]                        ;
        put     a16[8] <- 378                   ;
        putif   r128[5] <- 0xaf67937b5498dc     ;
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
        gt:u    a8[5],a64[9]                    ;
        lt:s    a8[5],a64[9]                    ;
        gt      r160[5],r256[9]                 ;
        lt      r160[5],r256[9]                 ;
        eq      a8[5],a64[9]                    ;
        eq      r160[5],r160[9]                 ;
        len     a512[6]                         ;
        cnt     a256[6]                         ;
        st2a                                    ;
        a2st                                    ;
        inc:c   a16[3]                          ;
        inc:u   a16[4],5                        ;
        dec:u   a16[3]                          ;
        dec:c   a16[4],5                        ;
        add:c   a32[12],a32[13]                 ;
        add:u   a32[12],a32[13]                 ;
        add:a   a32[12],a32[13]                 ;
        add:cs  a32[12],a32[13]                 ;
        add:us  a32[12],a32[13]                 ;
        add:as  a32[12],a32[13]                 ;
        add:f   a32[12],a32[13]                 ;
        add:af  a32[12],a32[13]                 ;
        sub:c   a32[12],a32[13]                 ;
        mul:c   a32[12],a32[13]                 ;
        div:c   a32[12],a32[13]                 ;
        rem:u   a64[8],a8[2]                    ;
        ecgen:secp r256[1],r512[1]              ;
        ecmul:secp a256[1],r512[1],r512[22]     ;
        ecadd:secp r512[22],r512[1]             ;
        ecneg:secp r512[1],r512[2]              ;
        ret                                     ;
        jmp     0                               ;
    };

    let lib = Lib::<NOp>::with(code).unwrap();
    println!("Serialization:\n{}\n", lib.bytecode().to_hex());
    let asm: Vec<Instr> = disassemble(lib.bytecode()).unwrap();
    println!("Assembly:");
    for instr in asm {
        println!("\t\t{}", instr);
    }

    print!("\nExecuting the program ... ");
    let mut runtime = Runtime::with(lib);
    match runtime.main() {
        Ok(true) => println!("success"),
        Ok(false) => println!("execution reported validation failure"),
        Err(err) => eprintln!("{}", err),
    }
}
