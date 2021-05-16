// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.


/// Example extension set of operations which are required for RGB
// TODO: Move to RGB Core Library
pub enum RgbOp {
    /// Counts number of metatdata of specific type
    CountMeta(u16, RegA, Reg32),
    CountState(u16, RegA, Reg32),
    CountRevealed(u16, RegA, Reg32),
    CountPublic(u16, RegA, Reg32),
    PullMeta(
        u16 /** State type */,
        Reg32 /** Value index from `a16` register */,
        Reg32 /** Destination start index */,
        Reg32 /** Destination end index. If smaller that start, indexes are switched */,
        bool /** Confidential or revealed */
    ),
    PullState(
        u16 /** State type */,
        Reg32 /** Value index from `a16` register */,
        Reg32 /** Destination start index */,
        Reg32 /** Destination end index. If smaller that start, indexes are switched */,
        bool /** Confidential or revealed */
    ),
    // We do not need the last two ops since they can be replaced with a library
    // operations utilizing AluVM byte string opcodes
    MatchMiniscript(
        u16 /** State type */,
        u16 /** Miniscript string length */,
        Box<[u8]> /** Miniscript template in strict encoded format */
    ),
    MatchPsbt(
        u16 /** State type */,
        u16 /** Psbt string length */,
        Box<[u8]> /** Psbt template in strict encoded format */
    )
}

fn main() {

}
