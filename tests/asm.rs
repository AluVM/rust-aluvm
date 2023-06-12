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
        put     12,a8[1];
        put     9,a8[2];
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, false);
}

#[test]
fn a8_eq() {
    let code = aluasm! {
        put     9,a8[1];
        put     9,a8[2];
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
        put     4,a16[1];
        put     4,a16[2];
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_eq_fail() {
    let code = aluasm! {
        put     3,a16[1];
        put     4,a16[2];
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
        put     2,a8[1];
        put     1,a8[2];
        gt.u    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_gt_s() {
    let code = aluasm! {
        put     1,a8[1];
        put     255,a8[2]; // -1
        gt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     1,a8[1];
        put     -1,a8[2];
        gt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     1,a8[1];
        put     2,a8[2];
        gt.s    a8[1],a8[2];
        ret;
    };
    run(code, false);
}

#[test]
fn a_lt_u() {
    let code = aluasm! {
        put     1,a8[1];
        put     2,a8[2];
        lt.u    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn a_lt_s() {
    let code = aluasm! {
        put     255,a8[1]; // -1
        put     1,a8[2];
        lt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     -1,a8[1];
        put     1,a8[2];
        lt.s    a8[1],a8[2];
        ret;
    };
    run(code, true);
    let code = aluasm! {
        put     2,a8[1];
        put     1,a8[2];
        lt.s    a8[1],a8[2];
        ret;
    };
    run(code, false);
}

#[test]
fn stp_add() {
    let code = aluasm! {
        put     3,a8[1];
        add     4,a8[1];
        put     7,a8[2];
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn stp_sub() {
    let code = aluasm! {
        put     3,a8[1];
        sub     4,a8[1];
        put     -1,a8[2];
        eq.n    a8[1],a8[2];
        ret;
    };
    run(code, true);
}

#[test]
fn float() {
    let code = aluasm! {
            put   1.25,f32[8];
            put   1.5,f32[9];
            put   2.75,f32[10];
            add.f f32[8],f32[9];
            eq.e  f32[9],f32[10];
            ret;
    };
    run(code, true);
}

#[test]
fn bytes_put() {
    let code = aluasm! {
            put   "aaa",s16[1];
            put   "aaa",s16[2];
            eq    s16[1],s16[2];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put   "aaa",s16[1];
            put   "bbb",s16[2];
            eq    s16[1],s16[2];
            ret;
    };
    run(code, false);
}

#[test]
fn bytes_extr() {
    let code = aluasm! {
            put    "################@@@@@@",s16[0];
            put    0,a16[0];
            extr   s16[0],r128[0],a16[0];
            put    0x23232323232323232323232323232323,r128[1];
            eq.n   r128[0],r128[1];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    "################@@@@@@",s16[0];
            put    3,a16[0];
            extr   s16[0],r128[0],a16[0];
            put    0x40404023232323232323232323232323,r128[1];
            eq.n   r128[0],r128[1];
            ret;
    };
    run(code, true);
}

#[test]
fn bytes_extr_offset_exceed() {
    let code = aluasm! {
            put    "123456788901234567",s16[0];
            put    0,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    "123456788901234567",s16[0];
            put    1,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    "123456788901234567",s16[0];
            put    2,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    "123456788901234567",s16[0];
            put    2,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    "################@",s16[0];
            put    1,a16[0];
            extr   s16[0],r128[0],a16[0];
            put    0x40232323232323232323232323232323,r128[1];
            eq.n   r128[0],r128[1];
            ret;
    };
    run(code, true);
    let code = aluasm! {
            put    "123456788901234567",s16[0];
            put    100,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    "123",s16[0];
            put    0,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, false);
}

#[test]
fn bytes_extr_uninitialized_offset() {
    let code = aluasm! {
            put    "12345678890123456",s16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    "12345678890123456",s16[0];
            extr   s16[0],r128[0],a16[0];
            eq.e   r128[0],r128[1];
            ret;
    };
    run(code, true);
}

#[test]
fn bytes_extr_uninitialized_bytes() {
    let code = aluasm! {
            put    0,a16[0];
            extr   s16[0],r128[0],a16[0];
            ret;
    };
    run(code, false);
    let code = aluasm! {
            put    0,a16[0];
            extr   s16[0],r128[0],a16[0];
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
