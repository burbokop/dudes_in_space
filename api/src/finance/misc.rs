use crate::finance::Bank;
use crate::finance::bank_registry::BankRegistry;
use crate::utils::math::NonNeg;
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Div, Mul};

pub type Currency = String;
pub type MoneyAmount = i64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Money {
    pub currency: Currency,
    pub amount: NonNeg<MoneyAmount>,
}

impl Mul<u32> for Money {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            currency: self.currency,
            amount: self.amount * rhs.into(),
        }
    }
}

impl Money {
    pub fn cmp(&self, bank_registry: &BankRegistry, other: &Money) -> Ordering {
        if self.currency == other.currency {
            return self.amount.cmp(&other.amount);
        }

        let other_amount = other
            .convert_to_currency(bank_registry, self.currency.clone())
            .amount
            .unwrap();
        let self_amount = self.amount.unwrap();

        self_amount.cmp(&other_amount)
    }

    pub fn min(self, bank_registry: &BankRegistry, other: Money) -> Money {
        let ord = self.cmp(bank_registry, &other);
        if ord == Ordering::Greater {
            other
        } else {
            self
        }
    }

    pub fn max(self, bank_registry: &BankRegistry, other: Money) -> Money {
        let ord = self.cmp(bank_registry, &other);
        if ord == Ordering::Less { other } else { self }
    }

    pub fn min_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        *self = self.clone().min(bank_registry, other);
    }

    pub fn max_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        *self = self.clone().max(bank_registry, other);
    }

    pub fn add(self, bank_registry: &BankRegistry, other: Money) -> Self {
        todo!()
    }
    pub fn sub(self, bank_registry: &BankRegistry, other: Money) -> Self {
        todo!()
    }

    pub fn add_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        todo!()
    }
    pub fn sub_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        todo!()
    }

    pub fn convert_to_currency(
        &self,
        bank_registry: &BankRegistry,
        target_currency: Currency,
    ) -> Money {
        todo!()
    }

    pub fn sum_as(
        bank_registry: &BankRegistry,
        iter: impl Iterator<Item = Money>,
        target_currency: Currency,
    ) -> Option<Money> {
        let amount: Vec<MoneyAmount> = iter
            .map(|money| {
                money
                    .convert_to_currency(bank_registry, target_currency.clone())
                    .amount
                    .unwrap()
            })
            .collect();

        if amount.is_empty() {
            return None;
        }

        Some(Money {
            currency: target_currency,
            amount: NonNeg::new(amount.into_iter().sum()).unwrap(),
        })
    }

    pub fn sum_same_currency(iter: impl Iterator<Item = Money>) -> Option<Money> {
        let v: Vec<_> = iter.collect();

        let currency = v.first()?.currency.clone();

        for money in v.iter() {
            // TODO: return error (not option because it is ambiguous)
            assert_eq!(money.currency, currency)
        }

        Some(Money {
            currency,
            amount: NonNeg::new(v.into_iter().map(|x| x.amount.unwrap()).sum()).unwrap(),
        })
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

impl Mul<Float> for Money {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self {
            currency: self.currency,
            amount: NonNeg::new((self.amount.unwrap() as Float * rhs) as MoneyAmount).unwrap(),
        }
    }
}

impl Div<Float> for Money {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        Self {
            currency: self.currency,
            amount: NonNeg::new((self.amount.unwrap() as Float / rhs) as MoneyAmount).unwrap(),
        }
    }
}
