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

use aluvm::isa::Instr;
use aluvm::libs::Lib;
use aluvm::Vm;

#[test]
fn a_eq_test() {
    let code = aluasm! {
        put     4,a16[1];
        put     4,a16[2];
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, true)
}

#[test]
fn a_eq_fail_test() {
    let code = aluasm! {
        put     3,a16[1];
        put     4,a16[2];
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, false)
}

#[test]
fn a_eq_noneeq_eq_test() {
    let code = aluasm! {
        eq.e    a16[1],a16[2];
        ret;
    };
    run(code, true)
}

#[test]
fn a_eq_noneeq_noneq_test() {
    let code = aluasm! {
        eq.n    a16[1],a16[2];
        ret;
    };
    run(code, false)
}

fn run(code: Vec<Instr>, expect_success: bool) {
    let mut runtime = Vm::<Instr>::with(Lib::assemble(&code).unwrap());
    let res = runtime.main();
    println!("\nVM microprocessor core state:\n{:#?}", runtime.registers());
    assert!(res == expect_success)
}
