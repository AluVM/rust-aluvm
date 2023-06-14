// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://github.com/internet2-org/aluvm-spec>
//
// Designed & written in 2021-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
// This work is donated to LNP/BP Standards Association by Pandora Core AG
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use aluvm::isa::Instr;
use aluvm::library::Lib;
use aluvm::{aluasm, Prog, Vm};

#[test]
fn a8_ne() {
    let code = aluasm! {
        put     a8[1],12;
        put     a8[2],9;
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, false);
}

#[test]
fn a8_eq() {
    let code = aluasm! {
        put     a8[1],9;
        put     a8[2],9;
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, false);
    let code = aluasm! {
        eq.e    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a16_eq() {
    let code = aluasm! {
        put     a16[1],4;
        put     a16[2],4;
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_eq_fail() {
    let code = aluasm! {
        put     a16[1],3;
        put     a16[2],4;
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, false);
}

#[test]
fn a_eq_noneeq_eq() {
    let code = aluasm! {
        eq.e    a16[1],a16[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_eq_noneeq_noneq() {
    let code = aluasm! {
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, false);
}

#[test]
fn a_gt_u() {
    let code = aluasm! {
        put     a8[1],2;
        put     a8[2],1;
        gt.u    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_gt_s() {
    let code = aluasm! {
        put     a8[1],1;
        put     a8[2],255; // -1
        gt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     a8[1],1;
        put     a8[2],-1;
        gt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     a8[1],1;
        put     a8[2],2;
        gt.s    a8[1],a8[2];
        ret;
    };
    run(code, false);
}

#[test]
fn a_lt_u() {
    let code = aluasm! {
        put     a8[1],1;
        put     a8[2],2;
        lt.u    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_lt_s() {
    let code = aluasm! {
        put     a8[1],255;
        put     a8[2],1;
        lt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     a8[1],-1;
        put     a8[2],1;
        lt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     a8[1],2;
        put     a8[2],1;
        lt.s    a8[1],a8[2];
        ret;
    };
    run(code, false);
}

#[test]
fn stp_add() {
    let code = aluasm! {
        put     a8[1],3;
        add     a8[1],4;
        put     a8[2],7;
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn stp_sub() {
    let code = aluasm! {
        put     a8[1],3;
        sub     a8[1],4;
        put     a8[2],-1;
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn float() {
    let code = aluasm! {
            put   f32[8],1.25;
            put   f32[9],1.5;
            put   f32[10],2.75;
            add.f f32[9],f32[8];
            eq.e  f32[9],f32[10];
            ret;
    };
    run(code, true);
}

#[test]
fn bytes_put() {
    let code = aluasm! {
            put   s16[1],"aaa";
            put   s16[2],"aaa";
            eq    s16[1],s16[2];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put   s16[1],"aaa";
            put   s16[2],"bbb";
            eq    s16[1],s16[2];
            ret;
    };
    run(code, false);
}

#[test]
fn bytes_extr() {
    let code = aluasm! {
            put    s16[0],"################@@@@@@";
            put    a16[0],0;
            extr   r128[0],s16[0],a16[0];
            put    r128[1],0x23232323232323232323232323232323;
            eq.n   r128[0],r128[1];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    s16[0],"################@@@@@@";
            put    a16[0],3;
            extr   r128[0],s16[0],a16[0];
            put    r128[1],0x40404023232323232323232323232323;
            eq.n   r128[0],r128[1];
            ret;
    };
    run(code, true);
}

#[test]
fn bytes_extr_offset_exceed() {
    let code = aluasm! {
            put    s16[0],"123456788901234567";
            put    a16[0],0;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    s16[0],"123456788901234567";
            put    a16[0],1;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    s16[0],"123456788901234567";
            put    a16[0],2;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    s16[0],"123456788901234567";
            put    a16[0],2;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    s16[0],"################@";
            put    a16[0],1;
            extr   r128[0],s16[0],a16[0];
            put    r128[1],0x40232323232323232323232323232323;
            eq.n   r128[0],r128[1];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    s16[0],"123456788901234567";
            put    a16[0],100;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    s16[0],"123";
            put    a16[0],0;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, false);
}

#[test]
fn bytes_extr_uninitialized_offset() {
    let code = aluasm! {
            put    s16[0],"12345678890123456";
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    s16[0],"12345678890123456";
            extr   r128[0],s16[0],a16[0];
            eq.e   r128[0],r128[1];
            ret;
    };
    run(code, true);
}

#[test]
fn bytes_extr_uninitialized_bytes() {
    let code = aluasm! {
            put    a16[0],0;
            extr   r128[0],s16[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    a16[0],0;
            extr   r128[0],s16[0],a16[0];
            eq.e   r128[0],r128[1];
            ret;
    };
    run(code, true);
}

fn run(code: Vec<Instr>, expect_success: bool) {
    let mut runtime = Vm::<Instr>::new();

    let program = Prog::<Instr>::new(Lib::assemble(&code).unwrap());
    let res = runtime.run(&program, &());

    println!("\nVM microprocessor core state:\n{:#?}", runtime.registers);
    assert!(res == expect_success);
}
