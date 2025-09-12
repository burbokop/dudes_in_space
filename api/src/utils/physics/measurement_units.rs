use crate::utils::math::{One, Rational};
use serde::{Deserialize, Serialize};
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Kg<T>(pub T);

/// m³
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct M3<T>(pub T);

impl Div for M3<f32> {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Div for M3<f64> {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Div for M3<u32> {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 as f32 / rhs.0 as f32
    }
}

impl Div for M3<u64> {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 as f64 / rhs.0 as f64
    }
}

impl<T> Add<Self> for M3<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T> Sub<Self> for M3<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<T> Sum for M3<T>
where
    T: Sum,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).sum())
    }
}

impl<T> Mul<T> for M3<T>
where
    T: Mul<T, Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<f32> for M3<u32> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self((self.0 as f32 * rhs) as u32)
    }
}

impl Mul<f64> for M3<u64> {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self((self.0 as f64 * rhs) as u64)
    }
}

impl<T: One> One for M3<T> {
    fn one() -> Self {
        M3(T::one())
    }
}

/// g/cm³
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct KgPerM3<T>(pub Rational<Kg<T>, M3<T>>);

impl<T> From<T> for KgPerM3<T>
where
    M3<T>: One,
{
    fn from(value: T) -> Self {
        KgPerM3(Rational::<Kg<T>, M3<T>>::from(Kg(value)))
    }
}
