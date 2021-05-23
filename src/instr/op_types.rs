// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::{u2, u3};
#[cfg(feature = "std")]
use std::fmt::{self, Display, Formatter};

/// Integer arithmetic types
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum NumType {
    /// Unsigned integer
    #[cfg_attr(feature = "std", display("u"))]
    Unsigned,

    /// Signed integer
    #[cfg_attr(feature = "std", display("s"))]
    Signed,

    /// Float number with 23-bit mantissa
    #[cfg_attr(feature = "std", display("f"))]
    Float23,

    /// Float number with 52 bit mantissa
    #[cfg_attr(feature = "std", display("d"))]
    Float52,
}

impl NumType {
    /// Constructs numeric type from `u2` value (used in bytecode serialization)
    pub fn from_u2(val: u2) -> NumType {
        match *val {
            0 => NumType::Unsigned,
            1 => NumType::Signed,
            2 => NumType::Float23,
            3 => NumType::Float52,
            _ => unreachable!(),
        }
    }

    /// Returns `u2` representation of numeric type (used in bytecode
    /// serialization).
    pub fn as_u2(self) -> u2 {
        match self {
            NumType::Unsigned => u2::with(0),
            NumType::Signed => u2::with(1),
            NumType::Float23 => u2::with(2),
            NumType::Float52 => u2::with(3),
        }
    }
}

impl From<u2> for NumType {
    fn from(val: u2) -> NumType {
        NumType::from_u2(val)
    }
}

impl From<&NumType> for u2 {
    fn from(nt: &NumType) -> u2 {
        nt.as_u2()
    }
}

impl From<NumType> for u2 {
    fn from(nt: NumType) -> u2 {
        nt.as_u2()
    }
}

/// Variants of arithmetic operations
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Arithmetics {
    IntChecked {
        /// Indicates the need to use signed integer arithmetics
        signed: bool,
    },
    IntUnchecked {
        /// Indicates the need to use signed integer arithmetics
        signed: bool,
    },
    IntArbitraryPrecision {
        /// Indicates the need to use signed integer arithmetics
        signed: bool,
    },
    Float,
    FloatArbitraryPrecision,
}

impl Arithmetics {
    /// Constructs arithmetics variant from `u3` value (used in bytecode
    /// serialization).
    pub fn from_u3(val: u3) -> Arithmetics {
        match *val {
            0 => Arithmetics::IntChecked { signed: false },
            1 => Arithmetics::IntUnchecked { signed: false },
            2 => Arithmetics::IntArbitraryPrecision { signed: false },
            3 => Arithmetics::IntChecked { signed: true },
            4 => Arithmetics::IntUnchecked { signed: true },
            5 => Arithmetics::IntArbitraryPrecision { signed: true },
            6 => Arithmetics::Float,
            7 => Arithmetics::FloatArbitraryPrecision,
            _ => unreachable!(),
        }
    }

    /// Returns `u3` representation of arithmetics variant (used in bytecode
    /// serialization).
    pub fn as_u3(self) -> u3 {
        match self {
            Arithmetics::IntChecked { signed: false } => u3::with(0),
            Arithmetics::IntUnchecked { signed: false } => u3::with(1),
            Arithmetics::IntArbitraryPrecision { signed: false } => u3::with(2),
            Arithmetics::IntChecked { signed: true } => u3::with(3),
            Arithmetics::IntUnchecked { signed: true } => u3::with(4),
            Arithmetics::IntArbitraryPrecision { signed: true } => u3::with(5),
            Arithmetics::Float => u3::with(6),
            Arithmetics::FloatArbitraryPrecision => u3::with(7),
        }
    }

    /// Detects arbitrary precision arithmetic operation type
    pub fn is_ap(self) -> bool {
        match self {
            Arithmetics::IntArbitraryPrecision { .. } | Arithmetics::FloatArbitraryPrecision => {
                true
            }
            _ => false,
        }
    }

    /// Detects float-based arithmetic operation type
    pub fn is_float(self) -> bool {
        match self {
            Arithmetics::Float | Arithmetics::FloatArbitraryPrecision => true,
            _ => false,
        }
    }

    /// Detects unsigned integer arithmetic operation type
    pub fn is_unsigned(self) -> bool {
        match self {
            Arithmetics::IntChecked { signed: false }
            | Arithmetics::IntUnchecked { signed: false }
            | Arithmetics::IntArbitraryPrecision { signed: false } => true,
            _ => false,
        }
    }
}

impl From<u3> for Arithmetics {
    fn from(val: u3) -> Arithmetics {
        Arithmetics::from_u3(val)
    }
}

impl From<&Arithmetics> for u3 {
    fn from(ar: &Arithmetics) -> u3 {
        ar.as_u3()
    }
}

impl From<Arithmetics> for u3 {
    fn from(ar: Arithmetics) -> u3 {
        ar.as_u3()
    }
}

#[cfg(feature = "std")]
impl Display for Arithmetics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Arithmetics::IntChecked { signed: false } => f.write_str("c"),
            Arithmetics::IntUnchecked { signed: false } => f.write_str("u"),
            Arithmetics::IntArbitraryPrecision { signed: false } => f.write_str("a"),
            Arithmetics::IntChecked { signed: true } => f.write_str("cs"),
            Arithmetics::IntUnchecked { signed: true } => f.write_str("us"),
            Arithmetics::IntArbitraryPrecision { signed: true } => f.write_str("as"),
            Arithmetics::Float => f.write_str("f"),
            Arithmetics::FloatArbitraryPrecision => f.write_str("af"),
        }
    }
}

/// Selector between increment and decrement operation
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "std", derive(Display))]
pub enum IncDec {
    /// Increment operation
    #[display("inc")]
    Inc,

    /// Decrement operation
    #[display("dec")]
    Dec,
}

impl IncDec {
    pub fn multiplier(self) -> i8 {
        match self {
            IncDec::Inc => 1,
            IncDec::Dec => -1,
        }
    }
}

impl From<bool> for IncDec {
    fn from(val: bool) -> IncDec {
        if val {
            IncDec::Dec
        } else {
            IncDec::Inc
        }
    }
}

impl From<&IncDec> for bool {
    fn from(val: &IncDec) -> bool {
        bool::from(*val)
    }
}

impl From<IncDec> for bool {
    fn from(val: IncDec) -> bool {
        val == IncDec::Dec
    }
}
