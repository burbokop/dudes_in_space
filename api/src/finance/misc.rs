use crate::finance::Bank;
use crate::finance::bank_registry::BankRegistry;
use crate::utils::math::NonNeg;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::Mul;

pub type Currency = String;
pub type MoneyAmount = i64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyRef {
    pub currency: Currency,
    pub amount: NonNeg<MoneyAmount>,
}

impl Mul<u32> for MoneyRef {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            currency: self.currency,
            amount: self.amount * rhs.into(),
        }
    }
}

impl MoneyRef {
    pub fn cmp(&self, other: &MoneyRef, reg: &BankRegistry) -> Ordering {
        todo!()
    }

    pub fn min(self, other: MoneyRef, reg: &BankRegistry) -> MoneyRef {
        todo!()
    }

    pub fn max(self, other: MoneyRef, reg: &BankRegistry) -> MoneyRef {
        todo!()
    }

    pub fn min_assign(&mut self, other: MoneyRef, reg: &BankRegistry) {
        todo!()
    }

    pub fn max_assign(&mut self, other: MoneyRef, reg: &BankRegistry) {
        todo!()
    }

    pub fn add(self, other: MoneyRef, reg: &BankRegistry) -> Self {
        todo!()
    }
    pub fn sub(self, other: MoneyRef, reg: &BankRegistry) -> Self {
        todo!()
    }

    pub fn add_assign(&mut self, other: MoneyRef, reg: &BankRegistry) {
        todo!()
    }
    pub fn sub_assign(&mut self, other: MoneyRef, reg: &BankRegistry) {
        todo!()
    }
}

#[derive(Clone)]
pub struct MoneyRefExt<'a> {
    pub currency_owner: &'a Bank,
    pub amount: NonNeg<MoneyAmount>,
}

impl<'a> PartialEq<Self> for MoneyRefExt<'a> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'a> Eq for MoneyRefExt<'a> {}

impl<'a> PartialOrd for MoneyRefExt<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.currency_owner
                .currency_price(other.currency_owner, self.amount)
                .cmp(&other.amount),
        )
    }
}

impl<'a> Ord for MoneyRefExt<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.currency_owner
            .currency_price(other.currency_owner, self.amount)
            .cmp(&other.amount)
    }
}
