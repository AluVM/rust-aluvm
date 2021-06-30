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

extern crate alloc;

#[macro_use]
extern crate aluvm;

#[macro_use]
extern crate paste;

use aluvm::isa::ReservedOp;
use aluvm::libs::Lib;
use aluvm::Vm;

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
        add:z   f32[12],f32[13]                 ;
        sub:n   f32[12],f32[13]                 ;
        mul:c   f32[12],f32[13]                 ;
        div:f   f32[12],f32[13]                 ;
        rem     a64[8],a8[2]                    ;
        inc     a16[3]                          ;
        add     5,a16[4]                        ;
        dec     a16[3]                          ;
        sub     7682,a16[4]                     ;
        neg     a64[16]                         ;
        abs     f128[11]                        ;
        and     a32[5],a32[6],a32[5]            ;
        xor     r128[5],r128[6],r128[5]         ;
        shr:u   a256[12],a16[2]                 ;
        shr:s   a256[12],a16[2]                 ;
        shl     r256[24],a16[22]                ;
        shr     r256[24],a16[22]                ;
        scr     r256[24],a16[22]                ;
        scl     r256[24],a16[22]                ;
        rev     a512[28]                        ;
        ripemd  s16[9],r160[7]                  ;
        sha2    s16[19],r256[1]                 ;
        secpgen r256[1],r512[1]                 ;
        secpmul a256[1],r512[1],r512[22]        ;
        secpadd r512[22],r512[1]                ;
        secpneg r512[1],r512[2]                 ;
        call    "alu1wnhusevxmdphv3dh8ada44k0xw66ahq9nzhkv39z07hmudhp380sq0dtml", 56 ;
        ret                                     ;
        jmp     0                               ;
    };

    println!("Instructions:\n{:#?}\n", code);
    let lib = Lib::<ReservedOp>::assemble(&code).unwrap();
    println!("\n{}\n", lib);
    let code = lib.disassemble().unwrap();
    println!("Assembly:");
    for instr in code {
        println!("\t\t{}", instr);
    }

    eprint!("\nExecuting the program {} ... ", lib.id());
    let mut runtime = Vm::with(lib);
    match runtime.main() {
        true => eprintln!("success"),
        false => eprintln!("failure"),
    }
}
