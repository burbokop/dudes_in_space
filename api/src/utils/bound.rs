use burbomath::{MinusOne, One, Zero};
use std::{
    error::Error,
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub trait IsInBound<T> {
    fn is_in_bound(v: &T) -> bool;
}

/// From 0 to 1
pub struct UnitInterval;

impl<T> IsInBound<T> for UnitInterval
where
    T: One + Zero + PartialOrd,
{
    fn is_in_bound(v: &T) -> bool {
        *v >= T::zero() && *v <= T::one()
    }
}

/// From -1 to 1
pub struct SymmetricUnitInterval;

impl<T> IsInBound<T> for SymmetricUnitInterval
where
    T: One + MinusOne + PartialOrd,
{
    fn is_in_bound(v: &T) -> bool {
        *v >= T::minus_one() && *v <= T::one()
    }
}

pub struct Bounded<T, B: IsInBound<T>> {
    value: T,
    _dp: PhantomData<B>,
}

#[derive(Debug)]
pub struct OutOfBoundError<T> {
    pub original_value: T,
}

impl<T: Debug + Display> Error for OutOfBoundError<T> {}

impl<T: Display> Display for OutOfBoundError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T, B: IsInBound<T>> Bounded<T, B> {
    pub fn new(value: T) -> Result<Self, OutOfBoundError<T>> {
        if B::is_in_bound(&value) {
            Ok(Self {
                value,
                _dp: Default::default(),
            })
        } else {
            Err(OutOfBoundError {
                original_value: value,
            })
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn inner(&self) -> &T {
        &self.value
    }
}
