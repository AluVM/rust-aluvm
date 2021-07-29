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

//! Internal data representations and operations on data used by AluVM

mod arithm;
mod bitwise;
mod byte_str;
#[cfg(feature = "std")]
pub mod encoding;
mod number;

pub use byte_str::ByteStr;
pub use number::{
    FloatLayout, IntLayout, Layout, LiteralParseError, MaybeNumber, Number, NumberLayout, Step,
};
