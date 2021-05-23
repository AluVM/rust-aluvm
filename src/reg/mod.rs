// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

mod arithm;
mod bitwise;
mod reg;
mod value;

pub use reg::{Reg, Reg16, Reg32, Reg8, RegA, RegBlock, RegR, Registers};
pub use value::{RegVal, Value};
