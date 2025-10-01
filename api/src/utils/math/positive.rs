use super::{Floor, IsPositive, NonNeg, Pi, Sqrt};
use serde::{Deserialize, Serialize};
use std::ops::{DivAssign, MulAssign, SubAssign};
use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, Mul, Sub},
};

/// Cannot store negative numbers
#[derive(Clone, Copy, Debug, Ord)]
pub struct Positive<T> {
    pub(super) value: T,
}

impl<T: Serialize> Serialize for Positive<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + IsPositive + Debug> Deserialize<'de> for Positive<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        if value.is_positive() {
            Ok(Self { value })
        } else {
            Err(serde::de::Error::custom(&format!(
                "Can not deserialize {:?} as Positive because it is not a positive number.",
                value
            )))
        }
    }
}

impl<T: Display> Display for Positive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

#[derive(Debug)]
pub struct NotPositiveError<T> {
    original_value: T,
}

impl<T> NotPositiveError<T> {
    pub(crate) fn original_value(self) -> T {
        self.original_value
    }
}

impl<T> Display for NotPositiveError<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T> Error for NotPositiveError<T> where T: Debug {}

impl<T> Positive<T> {
    pub fn new(value: T) -> Result<Self, NotPositiveError<T>>
    where
        T: IsPositive,
    {
        if value.is_positive() {
            Ok(Self { value })
        } else {
            Err(NotPositiveError {
                original_value: value,
            })
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    #[deprecated(note = "Better use NonNeg::into_inner")]
    pub fn unwrap(self) -> T {
        self.value
    }

    pub(crate) fn sqrt(self) -> Positive<<T as Sqrt>::Output>
    where
        T: Sqrt,
    {
        Positive {
            value: self.value.sqrt(),
        }
    }

    pub(crate) fn floor(self) -> Positive<<T as Floor>::Output>
    where
        T: Floor,
    {
        Positive {
            value: self.value.floor(),
        }
    }

    pub(crate) fn sub_assign(&mut self, rhs: Self) -> Result<(), Self>
    where
        T: SubAssign + PartialOrd,
    {
        if self.value > rhs.value {
            Ok(self.value -= rhs.value)
        } else {
            Err(rhs)
        }
    }
}

impl<T, U> PartialEq<Positive<U>> for Positive<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Positive<U>) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T, U> PartialEq<NonNeg<U>> for Positive<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &NonNeg<U>) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T> Eq for Positive<T>
where
    T: Eq,
{
    fn assert_receiver_is_total_eq(&self) {
        self.value.assert_receiver_is_total_eq()
    }
}

impl<T, U> PartialOrd<Positive<U>> for Positive<T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &Positive<U>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T, U> PartialOrd<NonNeg<U>> for Positive<T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &NonNeg<U>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T, U> Add<Positive<U>> for Positive<T>
where
    T: Add<U>,
{
    type Output = Positive<<T as Add<U>>::Output>;

    fn add(self, rhs: Positive<U>) -> Self::Output {
        Self::Output {
            value: self.value + rhs.value,
        }
    }
}

impl<T, U> Add<NonNeg<U>> for Positive<T>
where
    T: Add<U>,
{
    type Output = Positive<<T as Add<U>>::Output>;

    fn add(self, rhs: NonNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value + rhs.value,
        }
    }
}

impl<T, U> AddAssign<Positive<U>> for Positive<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: Positive<U>) {
        self.value += rhs.value
    }
}

impl<T, U> AddAssign<NonNeg<U>> for Positive<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: NonNeg<U>) {
        self.value += rhs.value
    }
}

impl<T, U> Sub<Positive<U>> for Positive<T>
where
    T: Sub<U>,
{
    type Output = <T as Sub<U>>::Output;

    fn sub(self, rhs: Positive<U>) -> Self::Output {
        self.value - rhs.value
    }
}

impl<T, U> Sub<NonNeg<U>> for Positive<T>
where
    T: Sub<U>,
{
    type Output = <T as Sub<U>>::Output;

    fn sub(self, rhs: NonNeg<U>) -> Self::Output {
        self.value - rhs.value
    }
}

impl<T, U> Mul<Positive<U>> for Positive<T>
where
    T: Mul<U>,
{
    type Output = Positive<<T as Mul<U>>::Output>;

    fn mul(self, rhs: Positive<U>) -> Self::Output {
        Self::Output {
            value: self.value * rhs.value,
        }
    }
}

impl<T, U> MulAssign<Positive<U>> for Positive<T>
where
    T: MulAssign<U>,
{
    fn mul_assign(&mut self, rhs: Positive<U>) {
        self.value *= rhs.value;
    }
}

impl<T, U> Div<Positive<U>> for Positive<T>
where
    T: Div<U>,
{
    type Output = Positive<<T as Div<U>>::Output>;

    fn div(self, rhs: Positive<U>) -> Self::Output {
        Self::Output {
            value: self.value / rhs.value,
        }
    }
}

impl<T, U> DivAssign<Positive<U>> for Positive<T>
where
    T: DivAssign<U>,
{
    fn div_assign(&mut self, rhs: Positive<U>) {
        self.value /= rhs.value;
    }
}

impl<T> Pi for Positive<T>
where
    T: Pi,
{
    fn pi() -> Self {
        Self { value: T::pi() }
    }
}

impl From<u32> for Positive<i64> {
    fn from(value: u32) -> Self {
        Self {
            value: value as i64,
        }
    }
}

impl<T: IsPositive> TryFrom<NonNeg<T>> for Positive<T> {
    type Error = NotPositiveError<T>;

    fn try_from(value: NonNeg<T>) -> Result<Self, Self::Error> {
        Self::new(value.value)
    }
}
