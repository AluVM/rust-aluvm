// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

extern crate alloc;

#[macro_use]
extern crate alure;

#[macro_use]
extern crate paste;

use core::convert::TryFrom;

use alure::instr::serialize::disassemble;
use alure::instr::{ArithmeticOp, CmpOp, ControlFlowOp, Instr, MoveOp, NOp, PutOp, Secp256k1Op};
use alure::{Lib, Reg16, Reg32, Reg8, RegA, RegAR, RegBlockAR, RegR, Runtime};
use amplify_num::hex::ToHex;
use amplify_num::u4;

fn main() {
    let code = aluasm! {
        clr     r1024[5]                        ;
        put     378,a16[8]                      ;
        putif   0xaf67937b5498dc,r128[5]        ;
        ecgen:secp r256[1],r512[1]              ;
        ecmul:secp a256[1],r512[1],r512[22]     ;
        ecadd:secp r512[22],r512[1]             ;
        ecneg:secp r512[1],r512[2]              ;
        ret                                     ;
        jmp     0                               ;
    };

    /*
       swp     a8[1], a16[2]                   ;
       swp     r256[8], r256[7]                ;
       swp     a256[1], r256[7]                ;
       mov     a8[1], a16[2]                   ;
       mov     r256[8], r256[7]                ;
       mov     a256[1], r256[7]                ;
       mov     r512[4], a512[3]                ;
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

    */

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
