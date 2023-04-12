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

//! Alu virtual machine

use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::isa::{Instr, InstructionSet, ReservedOp};
use crate::program::LibSite;
use crate::reg::CoreRegs;
use crate::Program;

/// Alu virtual machine providing single-core execution environment
#[derive(Getters, Debug, Default)]
pub struct Vm<Isa = Instr<ReservedOp>>
where
    Isa: InstructionSet,
{
    /// A set of registers
    registers: Box<CoreRegs>,

    phantom: PhantomData<Isa>,
}

/// Runtime for program execution.
impl<Isa> Vm<Isa>
where
    Isa: InstructionSet,
{
    /// Constructs new virtual machine instance.
    pub fn new() -> Self { Self { registers: Box::default(), phantom: Default::default() } }

    /// Executes the program starting from the provided entry point (set with
    /// [`Program::set_entrypoint`] and [`Program::with`], or initialized to 0 offset of the
    /// first used library if [`Program::new`] was used).
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    pub fn run<const MAX_LIBS: u16>(&mut self, program: &Program<Isa, MAX_LIBS>) -> bool {
        self.call(program, program.entrypoint())
    }

    /// Executes the program starting from the provided entry point.
    ///
    /// # Returns
    ///
    /// Value of the `st0` register at the end of the program execution.
    pub fn call<const MAX_LIBS: u16>(
        &mut self,
        program: &Program<Isa, MAX_LIBS>,
        method: LibSite,
    ) -> bool {
        let mut call = Some(method);
        while let Some(ref mut site) = call {
            if let Some(lib) = program.lib(site.lib) {
                call = lib.exec::<Isa>(site.pos, &mut self.registers);
            } else if let Some(pos) = site.pos.checked_add(1) {
                site.pos = pos;
            } else {
                call = None;
            };
        }
        self.registers.st0
    }
}
