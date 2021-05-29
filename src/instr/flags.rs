// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::fmt::{self, Display, Formatter, Write};
use core::str::FromStr;

use amplify_num::u1;

/// Errors for parsing string representation for a flag values
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[cfg_attr(feature = "std", derive(Error))]
#[display(doc_comments)]
pub enum ParseFlagError {
    /// Unknown operation flag `{0}`
    UnknownFlag(/** Unrecognized flag */ char),

    /// Unknown operation flags `{0}`
    UnknownFlags(/** Unrecognized flags */ String),

    /// Only one of mutually exclusive flags must be specified for {0} (only `{1}` or `{2}`)
    MutuallyExclusiveFlags(
        /** Flag description */ &'static str,
        /** Flag 1 */ char,
        /** Flag 2 */ char,
    ),

    /// Required flag for {0} is absent, please explicitly specify either `{1}` or `{2}`
    RequiredFlagAbsent(
        /** Flag description */ &'static str,
        /** Flag 1 */ char,
        /** Flag 2 */ char,
    ),
}

/// Integer encoding flag
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum SignFlag {
    /// Unsigned integer
    #[display("u")]
    Unsigned,

    /// Signed integer
    #[display("s")]
    Signed,
}

impl FromStr for SignFlag {
    type Err = ParseFlagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let filtered = s.replace(&['u', 's'], "");
        if !filtered.is_empty() {
            return Err(ParseFlagError::UnknownFlags(filtered));
        }
        match (s.contains('u'), s.contains('s')) {
            (true, false) => Ok(SignFlag::Unsigned),
            (false, true) => Ok(SignFlag::Signed),
            (true, true) => {
                Err(ParseFlagError::MutuallyExclusiveFlags("integer sign flag", 'u', 's'))
            }
            (false, false) => {
                Err(ParseFlagError::RequiredFlagAbsent("integer sign flag", 'u', 's'))
            }
        }
    }
}

impl SignFlag {
    /// Constructs integer sign flag from `u1` value (used in bytecode serialization)
    pub fn from_u1(val: u1) -> SignFlag {
        match val.as_u8() {
            0 => SignFlag::Unsigned,
            1 => SignFlag::Signed,
            _ => unreachable!(),
        }
    }

    /// Returns `u1` representation of integer sign flag (used in bytecode serialization).
    pub fn as_u1(self) -> u1 {
        match self {
            SignFlag::Unsigned => u1::with(0),
            SignFlag::Signed => u1::with(1),
        }
    }
}

impl From<u1> for SignFlag {
    fn from(val: u1) -> SignFlag { SignFlag::from_u1(val) }
}

impl From<&SignFlag> for u1 {
    fn from(nt: &SignFlag) -> u1 { nt.as_u1() }
}

impl From<SignFlag> for u1 {
    fn from(nt: SignFlag) -> u1 { nt.as_u1() }
}

/// Float equality flag
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum FloatEqFlag {
    /// Use exact match, when nearest floats are always non-equal.
    ///
    /// NB: This still implies `+0` == `-0`.
    #[display("e")]
    Exact,

    /// Use rounded matching, when floats which differ only on a single bit in significand are
    /// still treated as euqal.
    #[display("r")]
    Rounding,
}

impl FromStr for FloatEqFlag {
    type Err = ParseFlagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let filtered = s.replace(&['e', 'r'], "");
        if !filtered.is_empty() {
            return Err(ParseFlagError::UnknownFlags(filtered));
        }
        match (s.contains('e'), s.contains('r')) {
            (true, false) => Ok(FloatEqFlag::Exact),
            (false, true) => Ok(FloatEqFlag::Rounding),
            (true, true) => Err(ParseFlagError::MutuallyExclusiveFlags("float equality", 'e', 'r')),
            (false, false) => Err(ParseFlagError::RequiredFlagAbsent("float equality", 'e', 'r')),
        }
    }
}

impl FloatEqFlag {
    /// Constructs float comparison flag from `u1` value (used in bytecode serialization)
    pub fn from_u1(val: u1) -> FloatEqFlag {
        match val.as_u8() {
            0 => FloatEqFlag::Exact,
            1 => FloatEqFlag::Rounding,
            _ => unreachable!(),
        }
    }

    /// Returns `u1` representation of float comparison flag (used in bytecode serialization).
    pub fn as_u1(self) -> u1 {
        match self {
            FloatEqFlag::Exact => u1::with(0),
            FloatEqFlag::Rounding => u1::with(1),
        }
    }
}

impl From<u1> for FloatEqFlag {
    fn from(val: u1) -> FloatEqFlag { FloatEqFlag::from_u1(val) }
}

impl From<&FloatEqFlag> for u1 {
    fn from(nt: &FloatEqFlag) -> u1 { nt.as_u1() }
}

impl From<FloatEqFlag> for u1 {
    fn from(nt: FloatEqFlag) -> u1 { nt.as_u1() }
}

/// Rounding flags for float numbers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum RoundingFlag {
    /// Round to the nearest neighbour, and if the number is exactly in the middle, ties round to
    /// the nearest even digit in the required position.
    #[display("n")]
    TowardsNearest,

    /// Round always toward zero, which means ceiling for negative numbers and flooring for
    /// positive numbers.
    #[display("z")]
    TowardsZero,

    /// Round up (ceiling), ie toward +∞; negative results thus round toward zero.
    #[display("c")]
    Ceil,

    /// Round down (flooring), ie toward -∞; negative results thus round away from zero.
    #[display("f")]
    Floor,
}

impl FromStr for RoundingFlag {
    type Err = ParseFlagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let filtered = s.replace(&['n', 'z', 'c', 'f'], "");
        if !filtered.is_empty() {
            return Err(ParseFlagError::UnknownFlags(filtered));
        }
        if s.len() > 1 {
            return Err(ParseFlagError::MutuallyExclusiveFlags("float rounding", s[0], s[1]));
        }

        if s.contains('n') {
            Ok(RoundingFlag::TowardsNearest)
        } else if s.contains('z') {
            Ok(RoundingFlag::TowardsZero)
        } else if s.contains('c') {
            Ok(RoundingFlag::Ceil)
        } else if s.contains('f') {
            Ok(RoundingFlag::Floor)
        } else {
            Err(ParseFlagError::UnknownFlag(s[0]))
        }
    }
}

/// Encoding and overflowing flags for integer numbers
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct IntFlags {
    /// Treat the integer as signed (`true`) or unsigned (`false`). Signed integers has a different
    /// behaviour on detecting overflows, since they use only 7 bits for significant digits and not
    /// 8.
    pub signed: bool,

    /// Whether overflow must result in modulo-based wrapping (`true`) or set the destination into
    /// `None` state (`false`).
    pub wrap: bool,
}

impl Display for IntFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.signed {
            f.write_char('s')?;
        } else {
            f.write_char('u')?;
        }
        if self.wrap {
            f.write_char('w')
        } else {
            f.write_char('c')
        }
    }
}

impl FromStr for IntFlags {
    type Err = ParseFlagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let signed = match (s.contains('s'), s.contains('u')) {
            (true, false) => true,
            (false, true) => false,
            (true, true) => {
                return Err(ParseFlagError::MutuallyExclusiveFlags(
                    "integer serialization",
                    's',
                    'u',
                ))
            }
            (false, false) => {
                return Err(ParseFlagError::RequiredFlagAbsent("integer serialization", 's', 'u'))
            }
        };
        let wrap = match (s.contains('w'), s.contains('c')) {
            (true, false) => true,
            (false, true) => false,
            (true, true) => {
                return Err(ParseFlagError::MutuallyExclusiveFlags("overflow", 'w', 'c'))
            }
            (false, false) => return Err(ParseFlagError::RequiredFlagAbsent("overflow", 'w', 'c')),
        };
        if s.len() > 2 {
            return Err(ParseFlagError::UnknownFlags(s.replace(&['s', 'u', 'c', 'w'], "")));
        }

        Ok(IntFlags { signed, wrap })
    }
}
