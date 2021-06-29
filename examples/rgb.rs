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

use aluvm::{Reg32, RegA};

/// Example extension set of operations which are required for RGB
// TODO(#3) Move to RGB Core Library
pub enum RgbOp {
    /// Counts number of metatdata of specific type
    CountMeta(u16, RegA, Reg32),
    CountState(u16, RegA, Reg32),
    CountRevealed(u16, RegA, Reg32),
    CountPublic(u16, RegA, Reg32),
    PullMeta(
        /** State type */ u16,
        /** Value index from `a16` register */ Reg32,
        /** Destination start index */ Reg32,
        /** Destination end index. If smaller that start, indexes are
         * switched */
        Reg32,
        /** Confidential or revealed */ bool,
    ),
    PullState(
        /** State type */ u16,
        /** Value index from `a16` register */ Reg32,
        /** Destination start index */ Reg32,
        /** Destination end index. If smaller that start, indexes are
         * switched */
        Reg32,
        /** Confidential or revealed */ bool,
    ),
    // We do not need the last two ops since they can be replaced with a
    // library operations utilizing AluVM byte string opcodes
    MatchMiniscript(
        /** State type */ u16,
        /** Miniscript string length */ u16,
        /** Miniscript template in strict encoded format */ [u8; u64::MAX as usize],
    ),
    MatchPsbt(
        /** State type */ u16,
        /** Psbt string length */ u16,
        /** Psbt template in strict encoded format */ [u8; u64::MAX as usize],
    ),
}

fn main() {}
