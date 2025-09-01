use super::{Abs, Floor, IsNeg, Pi, Sqrt, Zero};
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};
use std::ops::{DivAssign, MulAssign, SubAssign};
use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, Mul, Sub},
};

/// Can not store negative numbers
#[derive(Clone, Copy, Debug, Ord)]
pub struct NonNeg<T> {
    value: T,
}

impl<T: Serialize> Serialize for NonNeg<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + IsNeg + Debug> Deserialize<'de> for NonNeg<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        if value.is_neg() {
            Err(serde::de::Error::custom(&format!(
                "Can not deserialize {:?} as NoNeg because it is negative.",
                value
            )))
        } else {
            Ok(Self { value })
        }
    }
}

impl<T: Display> Display for NonNeg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

#[derive(Debug)]
pub struct NegError<T> {
    original_value: T,
}

impl<T> NegError<T> {
    pub(crate) fn original_value(self) -> T {
        self.original_value
    }
}

impl<T> Display for NegError<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T> Error for NegError<T> where T: Debug {}

// impl<T> TryInto<NoNeg<T>> for T {
//     type Error = NegError;

//     fn try_into(self) -> Result<NoNeg<T>, Self::Error> {
//         todo!()
//     }
// }

impl<T> NonNeg<T> {
    #[deprecated(note = "Better use NonNeg::new")]
    pub(crate) fn wrap(value: T) -> Result<Self, NegError<T>>
    where
        T: IsNeg,
    {
        if value.is_neg() {
            Err(NegError {
                original_value: value,
            })
        } else {
            Ok(Self { value })
        }
    }

    pub fn new(value: T) -> Result<Self, NegError<T>>
    where
        T: IsNeg,
    {
        if value.is_neg() {
            Err(NegError {
                original_value: value,
            })
        } else {
            Ok(Self { value })
        }
    }

    pub fn unwrap(self) -> T {
        self.value
    }

    pub(crate) fn sqrt(self) -> NonNeg<<T as Sqrt>::Output>
    where
        T: Sqrt,
    {
        NonNeg {
            value: self.value.sqrt(),
        }
    }

    pub(crate) fn floor(self) -> NonNeg<<T as Floor>::Output>
    where
        T: Floor,
    {
        NonNeg {
            value: self.value.floor(),
        }
    }

    pub(crate) fn limited_sub<U>(self, rhs: NonNeg<U>) -> NonNeg<<T as Sub<U>>::Output>
    where
        T: Sub<U>,
        <T as Sub<U>>::Output: IsNeg,
        NonNeg<<T as Sub<U>>::Output>: Zero,
    {
        NonNeg::new(self.value - rhs.value).unwrap_or(Zero::zero())
    }

    pub(crate) fn sub_assign(&mut self, rhs: Self) -> Result<(), Self> where T: SubAssign + PartialOrd {
        if self.value >= rhs.value {
            Ok(self.value -= rhs.value)
        } else {
            Err(rhs)
        }
    }
}

impl<T, U> PartialEq<NonNeg<U>> for NonNeg<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &NonNeg<U>) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T> Eq for NonNeg<T>
where
    T: Eq,
{
    fn assert_receiver_is_total_eq(&self) {
        self.value.assert_receiver_is_total_eq()
    }
}

impl<T, U> PartialOrd<NonNeg<U>> for NonNeg<T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &NonNeg<U>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T, U> Add<NonNeg<U>> for NonNeg<T>
where
    T: Add<U>,
{
    type Output = NonNeg<<T as Add<U>>::Output>;

    fn add(self, rhs: NonNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value + rhs.value,
        }
    }
}

impl<T, U> AddAssign<NonNeg<U>> for NonNeg<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: NonNeg<U>) {
        self.value += rhs.value
    }
}

impl<T, U> Sub<NonNeg<U>> for NonNeg<T>
where
    T: Sub<U>,
{
    type Output = <T as Sub<U>>::Output;

    fn sub(self, rhs: NonNeg<U>) -> Self::Output {
        self.value - rhs.value
    }
}

impl<T, U> Mul<NonNeg<U>> for NonNeg<T>
where
    T: Mul<U>,
{
    type Output = NonNeg<<T as Mul<U>>::Output>;

    fn mul(self, rhs: NonNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value * rhs.value,
        }
    }
}

impl<T, U> MulAssign<NonNeg<U>> for NonNeg<T>
where
    T: MulAssign<U>,
{
    fn mul_assign(&mut self, rhs: NonNeg<U>) {
        self.value *= rhs.value;
    }
}

impl<T, U> Div<NonNeg<U>> for NonNeg<T>
where
    T: Div<U>,
{
    type Output = NonNeg<<T as Div<U>>::Output>;

    fn div(self, rhs: NonNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value / rhs.value,
        }
    }
}

impl<T, U> DivAssign<NonNeg<U>> for NonNeg<T>
where
    T: DivAssign<U>,
{
    fn div_assign(&mut self, rhs: NonNeg<U>) {
        self.value /= rhs.value;
    }
}

pub(crate) trait AbsAsNonNeg
where
    Self: Sized,
{
    type Output;
    fn abs_as_non_neg(self) -> NonNeg<Self::Output>;
}

impl<T> AbsAsNonNeg for T
where
    T: Abs,
{
    type Output = <T as Abs>::Output;
    fn abs_as_non_neg(self) -> NonNeg<Self::Output> {
        NonNeg { value: self.abs() }
    }
}

impl<T> Pi for NonNeg<T>
where
    T: Pi,
{
    fn pi() -> Self {
        Self { value: T::pi() }
    }
}

impl<T: Zero> Zero for NonNeg<T> {
    fn zero() -> Self {
        Self { value: T::zero() }
    }
}

impl<T: Zero> Default for NonNeg<T> { fn default() -> Self { Zero::zero() } }

impl From<u32> for NonNeg<i64> {
    fn from(value: u32) -> Self {
        Self {
            value: value as i64,
        }
    }
}

pub(crate) const fn noneg_f32(value: f32) -> NonNeg<f32> {
    assert!(value >= 0.);
    NonNeg { value }
}

pub(crate) const fn noneg_f64(value: f64) -> NonNeg<f64> {
    assert!(value >= 0.);
    NonNeg { value }
}

pub const fn noneg_float(value: Float) -> NonNeg<Float> {
    assert!(value >= 0.);
    NonNeg { value }
}

#[cfg(test)]
mod tests {
    use super::noneg_float;

    #[test]
    fn limited_sub() {
        assert_eq!(
            noneg_float(0.5).limited_sub(noneg_float(1.0)),
            noneg_float(0.0)
        );
        assert_eq!(
            noneg_float(1.0).limited_sub(noneg_float(0.5)),
            noneg_float(0.5)
        );
    }
}
