// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

extern crate alloc;

#[macro_use]
extern crate aluvm;

#[macro_use]
extern crate paste;

use core::convert::TryFrom;

use aluvm::instr::serialize::disassemble;
use aluvm::instr::{
    ArithmeticOp, CmpOp, ControlFlowOp, FloatEqFlag, Instr, IntFlags, MergeFlag, MoveOp, NOp,
    PutOp, Secp256k1Op, SignFlag,
};
use aluvm::{
    Lib, Reg16, Reg32, Reg8, RegA, RegAF, RegAR, RegBlockAFR, RegBlockAR, RegF, RegR, Runtime,
};
use amplify_num::hex::ToHex;
use amplify_num::{u3, u4};

fn main() {
    let code = aluasm! {
        clr     r1024[5]                        ;
        put     378,a16[8]                      ;
        putif   0xaf67937b5498dc,r128[5]        ;
        swp     a8[1],a8[2]                     ;
        swp     f256[8],f256[7]                 ;
        dup     a256[1],a256[7]                 ;
        mov     a16[1],a16[2]                   ;
        mov     r256[8],r256[7]                 ;
        cpy     a256[1],a256[7]                 ;
        cnv     f128[4],a128[3]                 ;
        spy     a1024[15],r1024[24]             ;
        gt:u    a8[5],a64[9]                    ;
        lt:s    a8[5],a64[9]                    ;
        gt:e    f64[5],f64[9]                   ;
        lt:r    f64[5],f64[9]                   ;
        gt      r160[5],r256[9]                 ;
        lt      r160[5],r256[9]                 ;
        eq      a8[5],a8[9]      .none=true     ;
        eq      r160[5],r160[9]  .none=false    ;
        eq:e    f64[19],f64[29]                 ;
        ifn     a32[32]                         ;
        ifz     r2048[17]                       ;
        inv     st0                             ;
        st:s    a8[1]                           ;
        add:uc  a32[12],a32[13]                 ;
        add:sw  a32[12],a32[13]                 ;
        sub:sc  a32[12],a32[13]                 ;
        mul:uw  a32[12],a32[13]                 ;
        div:cu  a32[12],a32[13]                 ;
        rem     a64[8],a8[2]                    ;
        secpgen r256[1],r512[1]                 ;
        secpmul a256[1],r512[1],r512[22]        ;
        secpadd r512[22],r512[1]                ;
        secpneg r512[1],r512[2]                 ;
        ret                                     ;
        jmp     0                               ;
    };

    /*
        add:z   f32[12],f32[13]                 ;
        add:n   f32[12],f32[13]                 ;
        len     a512[6]                         ;
        cnt     a256[6]                         ;
        st2a                                    ;
        a2st                                    ;
        inc:c   a16[3]                          ;
        inc:u   a16[4],5                        ;
        dec:u   a16[3]                          ;
        dec:c   a16[4],5                        ;
    */

    println!("Instructions:\n{:#?}\n", code);
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
