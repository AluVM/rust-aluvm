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

//! AluVM registers system

mod core_regs;
mod families;
mod indexes;

pub use core_regs::{CoreRegs, CALL_STACK_SIZE};
pub use families::{
    NumericRegister, RegA, RegA2, RegAF, RegAFR, RegAR, RegAll, RegBlock, RegBlockAFR, RegBlockAR,
    RegF, RegR,
};
pub use indexes::{Reg16, Reg32, Reg8, RegS};

/// Trait marking all types representing register family, specific register or register index
pub trait Register: Default {
    /// Text description of the register family
    fn description() -> &'static str;
}
