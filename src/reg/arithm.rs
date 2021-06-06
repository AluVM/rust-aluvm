// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::cmp::Ordering;
use core::ops::Add;

use half::bf16;
use rustc_apfloat::ieee;

use super::Number;
use crate::reg::number::{FloatLayout, Layout};

impl PartialEq for Number {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.layout() == other.layout()
            || self.layout().is_signed_int() && other.layout().is_unsigned_int()
            || self.layout().is_unsigned_int() && other.layout().is_signed_int())
            && self.to_clean().eq(&other.to_clean())
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

/// Since we always convert `NaN` values into `None` and keep them at the level of `MaybeNumber`, we
/// can do strict ordering even on float numbers
impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.layout(), other.layout(), "comparing numbers with different layout");
        match self.layout() {
            Layout::Integer(_) => match (self.is_positive(), other.is_positive()) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => self.to_u1024_bytes().cmp(&other.to_u1024_bytes()),
            },
            Layout::Float(FloatLayout::BFloat16) => {
                bf16::from(self).partial_cmp(&bf16::from(other)).expect("number value contains NaN")
            }
            Layout::Float(FloatLayout::IeeeHalf) => ieee::Half::from(self)
                .partial_cmp(&ieee::Half::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeSingle) => ieee::Single::from(self)
                .partial_cmp(&ieee::Single::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeDouble) => ieee::Double::from(self)
                .partial_cmp(&ieee::Double::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::X87DoubleExt) => ieee::X87DoubleExtended::from(self)
                .partial_cmp(&ieee::X87DoubleExtended::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeQuad) => ieee::Quad::from(self)
                .partial_cmp(&ieee::Quad::from(other))
                .expect("number value contains NaN"),
            Layout::Float(FloatLayout::IeeeOct) => {
                unimplemented!("IEEE-754 256-bit floats are not yet supported")
            }
            Layout::Float(FloatLayout::FloatTapered) => {
                unimplemented!("512-bit tapered floats are not yet supported")
            }
        }
    }
}

impl Number {
    /// Does comparison by ignoring the difference in the last bit of significand for float layouts.
    /// For integers performs normal comparison.
    pub fn rounding_cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.layout(), other.layout(), "comparing numbers with different layout");
        match self.layout() {
            Layout::Integer(_) => self.cmp(other),
            Layout::Float(FloatLayout::FloatTapered) => {
                unimplemented!("512-bit tapered floats are not yet supported")
            }
            Layout::Float(float_layout) => {
                let last_bit = Number::masked_bit(
                    float_layout
                        .significand_pos()
                        .expect("non-tapered float layout does not provides significand position")
                        .end,
                    self.layout(),
                );
                (*self ^ last_bit).cmp(&(*other ^ last_bit))
            }
        }
    }

    /// Checks for the equality ignoring the difference in the last bit of significand for float
    /// layouts. For integers performs normal comparison.
    #[inline]
    pub fn rounding_eq(&self, other: &Self) -> bool { self.rounding_cmp(other) == Ordering::Equal }
}

impl Add for Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output { todo!() }
}
