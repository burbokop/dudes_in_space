use std::ops::{Div, Mul, Rem, Sub};
use crate::utils::math::{Floor, One, Zero};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord,Clone)]
pub struct Rational<N, D> {
    pub numerator: N,
    pub denominator: D,
}

impl<N, D> From<N> for  Rational<N, D>
where D: One {
    fn from(value: N) -> Self {
        Self { numerator: value, denominator: D::one() }
    }
}

pub trait ApplyRationalPrecision<T> {
    fn apply_rational_precision(x: T) -> Self;
    fn precision() -> Self;
}

impl ApplyRationalPrecision<f32> for u32 {
    fn apply_rational_precision(x: f32) -> Self {
        (1000000000. * x).round() as Self
    }

    fn precision() -> Self {
        1000000000
    }
}

impl<T>  Rational<T, T>
{
    pub fn from_float<F>(value: F) -> Self
    where 
        T: ApplyRationalPrecision<F> + PartialOrd + Rem<Output=T> + Clone + Zero + Div<Output=T>,
        F: Floor<Output=F>+Clone + Sub<Output=F> + Mul<Output=F>
    {
            let frac = value.clone() - value.floor();

            let frac_mul_precision: T = ApplyRationalPrecision::apply_rational_precision(frac);
            let precision: T = ApplyRationalPrecision::precision();

            let gcd = greatest_common_divisor(frac_mul_precision.clone(), precision.clone());

            let denominator = precision / gcd.clone();
            let numerator = frac_mul_precision / gcd;

            Self { numerator, denominator }
    }
}

fn greatest_common_divisor<T>(a: T, b: T) -> T
where T: Zero + PartialOrd + Rem<Output = T> + Clone
{
    if a == T::zero() {
        b
    } else if b == T::zero() {
        a
    } else if a < b {
        greatest_common_divisor(a.clone(), b % a)
    }else {
        greatest_common_divisor(b.clone(), a % b)
    }
}