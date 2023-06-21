use std::fmt;

use crate::codec::error::DecodeError;
pub use crate::num_bigint::{BigInt, Sign};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum Number {
    FixInteger(FixInteger),
    Bignum(Bignum),
    Float(Float),
}
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Number::FixInteger(ref x) => x.fmt(f),
            Number::Bignum(ref x) => x.fmt(f),
            Number::Float(ref x) => x.fmt(f),
        }
    }
}
impl From<FixInteger> for Number {
    fn from(x: FixInteger) -> Self {
        Number::FixInteger(x)
    }
}
impl From<Bignum> for Number {
    fn from(x: Bignum) -> Self {
        Number::Bignum(x)
    }
}
impl From<Float> for Number {
    fn from(x: Float) -> Self {
        Number::Float(x)
    }
}

/// Fixed-size 32-bit signed integer.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct FixInteger {
    /// The value of the fixed-size integer.
    pub value: i32,
}
impl fmt::Display for FixInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl From<i8> for FixInteger {
    fn from(value: i8) -> Self {
        FixInteger {
            value: i32::from(value),
        }
    }
}
impl From<u8> for FixInteger {
    fn from(value: u8) -> Self {
        FixInteger {
            value: i32::from(value),
        }
    }
}
impl From<i16> for FixInteger {
    fn from(value: i16) -> Self {
        FixInteger {
            value: i32::from(value),
        }
    }
}
impl From<u16> for FixInteger {
    fn from(value: u16) -> Self {
        FixInteger {
            value: i32::from(value),
        }
    }
}
impl From<i32> for FixInteger {
    fn from(value: i32) -> Self {
        FixInteger { value }
    }
}

/// Big number (larger than unsigned 32-bit integer).
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Bignum {
    /// The value of the bignum.
    pub value: BigInt,
}
impl fmt::Display for Bignum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl From<i8> for Bignum {
    fn from(value: i8) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<u8> for Bignum {
    fn from(value: u8) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<i16> for Bignum {
    fn from(value: i16) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<u16> for Bignum {
    fn from(value: u16) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<i32> for Bignum {
    fn from(value: i32) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<u32> for Bignum {
    fn from(value: u32) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<i64> for Bignum {
    fn from(value: i64) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<u64> for Bignum {
    fn from(value: u64) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<i128> for Bignum {
    fn from(value: i128) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<u128> for Bignum {
    fn from(value: u128) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<isize> for Bignum {
    fn from(value: isize) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl From<usize> for Bignum {
    fn from(value: usize) -> Self {
        Bignum {
            value: BigInt::from(value),
        }
    }
}
impl<'a> From<&'a FixInteger> for Bignum {
    fn from(i: &'a FixInteger) -> Self {
        Bignum {
            value: BigInt::from(i.value),
        }
    }
}

/// 64-bit floating point number.
#[derive(Clone, Debug)]
pub struct Float {
    /// The value of the 64-bit floating point number.
    pub value: f64,
}
impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl TryFrom<f32> for Float {
    type Error = DecodeError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Float {
                value: f64::from(value),
            })
        } else {
            Err(DecodeError::NonFiniteFloat)
        }
    }
}
impl TryFrom<f64> for Float {
    type Error = DecodeError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Float { value })
        } else {
            Err(DecodeError::NonFiniteFloat)
        }
    }
}
impl Eq for Float {}
impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        ordered_float::OrderedFloat(self.value).eq(&ordered_float::OrderedFloat(other.value))
    }
}
impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        ordered_float::OrderedFloat(self.value)
            .partial_cmp(&ordered_float::OrderedFloat(other.value))
    }
}
impl std::hash::Hash for Float {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ordered_float::OrderedFloat(self.value).hash(state)
    }
}
