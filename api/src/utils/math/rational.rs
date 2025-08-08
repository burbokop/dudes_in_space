use std::ops::{Div, Rem};
use crate::utils::math::Zero;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Rational<T> {
    pub numerator: T,
    pub denominator: T,
}

impl<T> Rational<T> {
    fn from_f32(v: f32) -> Result< Self ,<T as TryFrom<f32>>::Error>
        where T: TryFrom<f32> + PartialOrd + Rem<Output=T> + Clone + Zero + Div<Output=T>
    {
        static PRECISION: f32 = 1000000000.;
        let frac = v - v.floor();

        let frac_mul_precision: T = TryFrom::try_from((frac * PRECISION).round())?;
        let precision: T = TryFrom::try_from(PRECISION)?;

        let gcd = greatest_common_divisor(frac_mul_precision.clone(), precision.clone());

        let denominator = precision / gcd.clone();
        let numerator = frac_mul_precision / gcd;

        Ok(Self { numerator, denominator })
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