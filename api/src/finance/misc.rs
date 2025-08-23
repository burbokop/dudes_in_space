use crate::finance::Bank;
use crate::finance::bank_registry::BankRegistry;
use crate::utils::math::NoNeg;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub type Currency = String;
pub type MoneyAmount = i64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyRef {
    pub currency: Currency,
    pub amount: NoNeg<MoneyAmount>,
}

impl MoneyRef {
    pub fn cmp(&self, other: &MoneyRef, reg: &BankRegistry) -> Ordering {
        todo!()
    }
}

#[derive(Clone)]
pub struct MoneyRefExt<'a> {
    pub currency_owner: &'a Bank,
    pub amount: NoNeg<MoneyAmount>,
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
