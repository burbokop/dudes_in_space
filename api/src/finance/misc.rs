use crate::finance::bank_registry::BankRegistry;
use crate::utils::math::NonNeg;
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Display;
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
    pub fn cmp_same_currency(&self, other: &Money) -> Ordering {
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        struct Impl<'a> {
            pub currency: &'a Currency,
            pub amount: &'a NonNeg<MoneyAmount>,
        }

        Impl {
            currency: &self.currency,
            amount: &self.amount,
        }
        .cmp(&Impl {
            currency: &other.currency,
            amount: &other.amount,
        })
    }

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
        let bank_registry = bank_registry.borrow();
        let this_bank = bank_registry.bank(&self.currency).unwrap();
        let target_bank = bank_registry.bank(&target_currency).unwrap();
        let target_amount = this_bank.sell_this_currency_price(&target_bank, self.amount);

        Self {
            currency: target_currency,
            amount: target_amount,
        }
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

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.currency.chars().count() > 1 {
            write!(f, "{} {}", self.amount, self.currency)
        } else {
            write!(f, "{}{}", self.amount, self.currency)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::{Bank, Wallet};
    use crate::person::PersonId;

    #[test]
    fn convert_to_currency_test() {
        let reg = BankRegistry::new();

        let create_bank = |created_money: Money| {
            let owner = PersonId::new_v4();
            let mut owner_wallet = Wallet::new();
            let mut bank = Bank::new(owner, created_money.currency);

            bank.withdraw(owner, &mut owner_wallet, created_money.amount)
                .unwrap();
            reg.register(bank).unwrap();
        };

        create_bank(Money {
            currency: "USD".to_string(),
            amount: 100000.into(),
        });
        create_bank(Money {
            currency: "EUR".to_string(),
            amount: 10000.into(),
        });
        create_bank(Money {
            currency: "GBP".to_string(),
            amount: 1000.into(),
        });

        let test_money_gbp = Money {
            currency: "GBP".to_string(),
            amount: 10.into(),
        };

        let test_money_usd = test_money_gbp.convert_to_currency(&reg, "USD".to_string());
        let test_money_eur = test_money_usd.convert_to_currency(&reg, "EUR".to_string());
        let intermediate_test_money_gbp =
            test_money_usd.convert_to_currency(&reg, "GBP".to_string());
        let final_test_money_gbp = test_money_eur.convert_to_currency(&reg, "GBP".to_string());

        assert!(
            test_money_gbp
                .cmp_same_currency(&intermediate_test_money_gbp)
                .is_eq()
        );
        assert!(
            test_money_gbp
                .cmp_same_currency(&final_test_money_gbp)
                .is_eq()
        );
    }
}
