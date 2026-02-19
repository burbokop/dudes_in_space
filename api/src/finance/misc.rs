use crate::finance::TargetBankDidNotCreateAnyMoneyError;
use crate::finance::bank_registry::BankRegistry;
use crate::utils::utils::Float;
use burbomath::math::{NegError, NonNeg, Positive, Zero};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Display;
use std::ops::{Div, Mul};

pub type Currency = String;
pub type MoneyAmount = i64;

#[macro_export]
macro_rules! money {
    ($amount:literal, $currency:literal) => {
        dudes_in_space_api::finance::Money {
            currency: $currency.into(),
            amount: dudes_in_space_api::utils::math::NonNeg::new($amount).unwrap(),
        }
    };
}

/// TODO remove and make money! return PossiblyNegativeMoney if amount literal is negative
#[macro_export]
macro_rules! possibly_negative_money {
    ($amount:literal, $currency:literal) => {
        dudes_in_space_api::finance::PossiblyNegativeMoney {
            currency: $currency.into(),
            amount: $amount,
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Money {
    pub currency: Currency,
    pub amount: NonNeg<MoneyAmount>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PossiblyNegativeMoney {
    pub currency: Currency,
    pub amount: MoneyAmount,
}

impl TryFrom<PossiblyNegativeMoney> for Money {
    type Error = NegError<MoneyAmount>;

    fn try_from(value: PossiblyNegativeMoney) -> Result<Self, Self::Error> {
        Ok(Self {
            currency: value.currency,
            amount: NonNeg::new(value.amount)?,
        })
    }
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

impl Mul<u32> for PossiblyNegativeMoney {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            currency: self.currency,
            amount: self.amount * rhs as MoneyAmount,
        }
    }
}

impl Money {
    pub fn zero(currency: Currency) -> Self {
        Self {
            currency,
            amount: Zero::zero(),
        }
    }

    pub fn mul_ceil(self, f: Float) -> Self {
        Self {
            currency: self.currency,
            amount: NonNeg::new((self.amount.into_inner() as Float * f).ceil() as MoneyAmount)
                .unwrap(),
        }
    }

    pub fn cmp_same_currency(&self, other: &Money) -> Result<Ordering, DifferentCurrenciesError> {
        if self.currency != other.currency {
            return Err(DifferentCurrenciesError);
        }

        Ok(self.amount.cmp(&other.amount))
    }

    pub fn cmp(&self, bank_registry: &BankRegistry, other: &Money) -> Ordering {
        if self.currency == other.currency {
            return self.amount.cmp(&other.amount);
        }

        let other_amount = other
            .convert_to_currency(bank_registry, self.currency.clone())
            .unwrap()
            .amount
            .into_inner();
        let self_amount = self.amount.into_inner();

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

    pub fn min_same_currency(self, other: Money) -> Result<Self, DifferentCurrenciesError> {
        let ord = self.cmp_same_currency(&other)?;
        Ok(if ord == Ordering::Greater {
            other
        } else {
            self
        })
    }

    pub fn max_same_currency(self, other: Money) -> Result<Self, DifferentCurrenciesError> {
        let ord = self.cmp_same_currency(&other)?;
        Ok(if ord == Ordering::Less { other } else { self })
    }

    pub fn min_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        *self = self.clone().min(bank_registry, other);
    }

    pub fn max_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        *self = self.clone().max(bank_registry, other);
    }

    pub fn add(self, bank_registry: &BankRegistry, other: Money) -> Self {
        if self.currency == other.currency {
            return Self {
                currency: self.currency,
                amount: self.amount + other.amount,
            };
        }

        let other_amount = other
            .convert_to_currency(bank_registry, self.currency.clone())
            .unwrap()
            .amount;

        Self {
            currency: self.currency,
            amount: self.amount + other_amount,
        }
    }

    pub fn sub(self, bank_registry: &BankRegistry, other: Money) -> PossiblyNegativeMoney {
        if self.currency == other.currency {
            return PossiblyNegativeMoney {
                currency: self.currency,
                amount: self.amount - other.amount,
            };
        }

        let other_amount = other
            .convert_to_currency(bank_registry, self.currency.clone())
            .unwrap()
            .amount
            .into_inner();
        let self_amount = self.amount.into_inner();

        PossiblyNegativeMoney {
            currency: self.currency,
            amount: self_amount - other_amount,
        }
    }

    pub fn add_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        *self = self.clone().add(bank_registry, other);
    }

    pub fn sub_assign(&mut self, bank_registry: &BankRegistry, other: Money) {
        todo!()
    }

    pub fn add_same_currency(self, other: Money) -> Result<Self, DifferentCurrenciesError> {
        todo!()
    }

    pub fn sub_same_currency(
        self,
        other: Money,
    ) -> Result<PossiblyNegativeMoney, DifferentCurrenciesError> {
        if self.currency != other.currency {
            return Err(DifferentCurrenciesError);
        }

        Ok(PossiblyNegativeMoney {
            currency: self.currency,
            amount: self.amount - other.amount,
        })
    }

    pub fn add_assign_same_currency(
        &mut self,
        other: Money,
    ) -> Result<(), DifferentCurrenciesError> {
        if self.currency != other.currency {
            return Err(DifferentCurrenciesError);
        }

        Ok(self.amount += other.amount)
    }

    pub fn sub_assign_same_currency(
        &mut self,
        other: Money,
    ) -> Result<(), DifferentCurrenciesError> {
        todo!()
    }

    pub fn convert_to_currency(
        &self,
        bank_registry: &BankRegistry,
        target_currency: Currency,
    ) -> Result<Money, TargetBankDidNotCreateAnyMoneyError> {
        let bank_registry = bank_registry.borrow();
        let this_bank = bank_registry.bank(&self.currency).unwrap();
        let target_bank = bank_registry.bank(&target_currency).unwrap();
        let target_amount = this_bank.sell_this_currency_price(&target_bank, self.amount)?;

        Ok(Self {
            currency: target_currency,
            amount: target_amount,
        })
    }

    pub fn sum_as(
        bank_registry: &BankRegistry,
        iter: impl Iterator<Item = Money>,
        target_currency: Currency,
    ) -> Result<Option<Money>, TargetBankDidNotCreateAnyMoneyError> {
        let amount: Result<Vec<MoneyAmount>, TargetBankDidNotCreateAnyMoneyError> = iter
            .map(|money| {
                Ok(money
                    .convert_to_currency(bank_registry, target_currency.clone())?
                    .amount
                    .into_inner())
            })
            .collect();

        let amount = amount?;
        if amount.is_empty() {
            return Ok(None);
        }

        Ok(Some(Money {
            currency: target_currency,
            amount: NonNeg::new(amount.into_iter().sum()).unwrap(),
        }))
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
            amount: NonNeg::new(v.into_iter().map(|x| x.amount.into_inner()).sum()).unwrap(),
        })
    }

    pub fn try_sum_same_currency<E>(
        iter: impl Iterator<Item = Result<Money, E>>,
    ) -> Result<Option<Money>, E> {
        let v: Result<Vec<_>, _> = iter.collect();
        let v = v?;

        let currency = match v.first() {
            None => unreachable!(),
            Some(money) => money.currency.clone(),
        };

        for money in v.iter() {
            // TODO: return error (not option because it is ambiguous)
            assert_eq!(money.currency, currency)
        }

        Ok(Some(Money {
            currency,
            amount: NonNeg::new(v.into_iter().map(|x| x.amount.into_inner()).sum()).unwrap(),
        }))
    }
}

impl PossiblyNegativeMoney {
    pub fn zero(currency: Currency) -> Self {
        Self {
            currency,
            amount: Zero::zero(),
        }
    }

    pub fn cmp_same_currency(&self, other: &Self) -> Result<Ordering, DifferentCurrenciesError> {
        if self.currency != other.currency {
            return Err(DifferentCurrenciesError);
        }

        Ok(self.amount.cmp(&other.amount))
    }

    pub fn cmp(&self, bank_registry: &BankRegistry, other: &Self) -> Ordering {
        if self.currency == other.currency {
            return self.amount.cmp(&other.amount);
        }

        let other_amount = other
            .convert_to_currency(bank_registry, self.currency.clone())
            .unwrap()
            .amount;
        let self_amount = self.amount;

        self_amount.cmp(&other_amount)
    }

    pub fn min(self, bank_registry: &BankRegistry, other: Self) -> Self {
        let ord = self.cmp(bank_registry, &other);
        if ord == Ordering::Greater {
            other
        } else {
            self
        }
    }

    pub fn max(self, bank_registry: &BankRegistry, other: Self) -> Self {
        let ord = self.cmp(bank_registry, &other);
        if ord == Ordering::Less { other } else { self }
    }

    pub fn min_same_currency(self, other: Self) -> Result<Self, DifferentCurrenciesError> {
        let ord = self.cmp_same_currency(&other)?;
        Ok(if ord == Ordering::Greater {
            other
        } else {
            self
        })
    }

    pub fn max_same_currency(self, other: Self) -> Result<Self, DifferentCurrenciesError> {
        let ord = self.cmp_same_currency(&other)?;
        Ok(if ord == Ordering::Less { other } else { self })
    }

    pub fn min_assign(&mut self, bank_registry: &BankRegistry, other: Self) {
        *self = self.clone().min(bank_registry, other);
    }

    pub fn max_assign(&mut self, bank_registry: &BankRegistry, other: Self) {
        *self = self.clone().max(bank_registry, other);
    }

    pub fn add(self, bank_registry: &BankRegistry, other: Self) -> Self {
        todo!()
    }

    pub fn sub(self, bank_registry: &BankRegistry, other: Self) -> Self {
        todo!()
    }

    pub fn add_assign(&mut self, bank_registry: &BankRegistry, other: Self) {
        todo!()
    }

    pub fn sub_assign(&mut self, bank_registry: &BankRegistry, other: Self) {
        todo!()
    }

    pub fn add_same_currency(self, other: Self) -> Result<Self, DifferentCurrenciesError> {
        todo!()
    }

    pub fn sub_same_currency(self, other: Self) -> Result<Self, DifferentCurrenciesError> {
        if self.currency != other.currency {
            return Err(DifferentCurrenciesError);
        }

        Ok(Self {
            currency: self.currency,
            amount: self.amount - other.amount,
        })
    }

    pub fn add_assign_same_currency(
        &mut self,
        other: Self,
    ) -> Result<(), DifferentCurrenciesError> {
        todo!()
    }

    pub fn sub_assign_same_currency(
        &mut self,
        other: Self,
    ) -> Result<(), DifferentCurrenciesError> {
        todo!()
    }

    pub fn convert_to_currency(
        &self,
        bank_registry: &BankRegistry,
        target_currency: Currency,
    ) -> Result<Self, TargetBankDidNotCreateAnyMoneyError> {
        let bank_registry = bank_registry.borrow();
        let this_bank = bank_registry.bank(&self.currency).unwrap();
        let target_bank = bank_registry.bank(&target_currency).unwrap();
        let target_amount =
            this_bank.sell_this_currency_price_possibly_negative(&target_bank, self.amount)?;

        Ok(Self {
            currency: target_currency,
            amount: target_amount,
        })
    }

    pub fn sum_as(
        bank_registry: &BankRegistry,
        iter: impl Iterator<Item = Self>,
        target_currency: Currency,
    ) -> Result<Option<Self>, TargetBankDidNotCreateAnyMoneyError> {
        let amount: Result<Vec<MoneyAmount>, TargetBankDidNotCreateAnyMoneyError> = iter
            .map(|money| {
                Ok(money
                    .convert_to_currency(bank_registry, target_currency.clone())?
                    .amount)
            })
            .collect();

        let amount = amount?;
        if amount.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self {
            currency: target_currency,
            amount: amount.into_iter().sum(),
        }))
    }

    pub fn sum_same_currency(iter: impl Iterator<Item = Self>) -> Option<Self> {
        let v: Vec<_> = iter.collect();

        let currency = v.first()?.currency.clone();

        for money in v.iter() {
            // TODO: return error (not option because it is ambiguous)
            assert_eq!(money.currency, currency)
        }

        Some(Self {
            currency,
            amount: v.into_iter().map(|x| x.amount).sum(),
        })
    }

    pub fn try_sum_same_currency<E>(
        iter: impl Iterator<Item = Result<Self, E>>,
    ) -> Result<Option<Self>, E> {
        let v: Result<Vec<_>, _> = iter.collect();
        let v = v?;

        let currency = match v.first() {
            None => unreachable!(),
            Some(money) => money.currency.clone(),
        };

        for money in v.iter() {
            // TODO: return error (not option because it is ambiguous)
            assert_eq!(money.currency, currency)
        }

        Ok(Some(Self {
            currency,
            amount: v.into_iter().map(|x| x.amount).sum(),
        }))
    }
}

impl Mul<Float> for Money {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self {
            currency: self.currency,
            amount: NonNeg::new((self.amount.into_inner() as Float * rhs) as MoneyAmount).unwrap(),
        }
    }
}

impl Mul<NonNeg<Float>> for Money {
    type Output = Self;

    fn mul(self, rhs: NonNeg<Float>) -> Self::Output {
        Self {
            currency: self.currency,
            amount: NonNeg::new(
                (self.amount.into_inner() as Float * rhs.into_inner()) as MoneyAmount,
            )
            .unwrap(),
        }
    }
}

impl Mul<Positive<Float>> for Money {
    type Output = Self;

    fn mul(self, rhs: Positive<Float>) -> Self::Output {
        Self {
            currency: self.currency,
            amount: NonNeg::new(
                (self.amount.into_inner() as Float * rhs.into_inner()) as MoneyAmount,
            )
            .unwrap(),
        }
    }
}

impl Div<Float> for Money {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        Self {
            currency: self.currency,
            amount: NonNeg::new((self.amount.into_inner() as Float / rhs) as MoneyAmount).unwrap(),
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

impl Mul<Float> for PossiblyNegativeMoney {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self {
            currency: self.currency,
            amount: (self.amount as Float * rhs) as MoneyAmount,
        }
    }
}

impl Mul<NonNeg<Float>> for PossiblyNegativeMoney {
    type Output = Self;

    fn mul(self, rhs: NonNeg<Float>) -> Self::Output {
        Self {
            currency: self.currency,
            amount: (self.amount as Float * rhs.into_inner()) as MoneyAmount,
        }
    }
}

impl Mul<Positive<Float>> for PossiblyNegativeMoney {
    type Output = Self;

    fn mul(self, rhs: Positive<Float>) -> Self::Output {
        Self {
            currency: self.currency,
            amount: (self.amount as Float * rhs.into_inner()) as MoneyAmount,
        }
    }
}

impl Div<Float> for PossiblyNegativeMoney {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        Self {
            currency: self.currency,
            amount: (self.amount as Float / rhs) as MoneyAmount,
        }
    }
}

impl Display for PossiblyNegativeMoney {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.currency.chars().count() > 1 {
            write!(f, "{} {}", self.amount, self.currency)
        } else {
            write!(f, "{}{}", self.amount, self.currency)
        }
    }
}

#[derive(Debug)]
pub struct DifferentCurrenciesError;

impl Display for DifferentCurrenciesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for DifferentCurrenciesError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::WalletRegistry;
    use crate::finance::tests::create_bank;

    #[test]
    fn convert_to_currency_test() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 100000.into(),
            },
        );
        create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "EUR".to_string(),
                amount: 10000.into(),
            },
        );
        create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "GBP".to_string(),
                amount: 1000.into(),
            },
        );

        let test_money_gbp = Money {
            currency: "GBP".to_string(),
            amount: 10.into(),
        };

        let test_money_usd = test_money_gbp
            .convert_to_currency(&bank_registry, "USD".to_string())
            .unwrap();
        let test_money_eur = test_money_usd
            .convert_to_currency(&bank_registry, "EUR".to_string())
            .unwrap();
        let intermediate_test_money_gbp = test_money_usd
            .convert_to_currency(&bank_registry, "GBP".to_string())
            .unwrap();
        let final_test_money_gbp = test_money_eur
            .convert_to_currency(&bank_registry, "GBP".to_string())
            .unwrap();

        assert!(
            test_money_gbp
                .cmp_same_currency(&intermediate_test_money_gbp)
                .unwrap()
                .is_eq()
        );
        assert!(
            test_money_gbp
                .cmp_same_currency(&final_test_money_gbp)
                .unwrap()
                .is_eq()
        );
    }

    #[test]
    fn convert_negative_to_currency_test() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 100000.into(),
            },
        );
        create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "EUR".to_string(),
                amount: 10000.into(),
            },
        );
        create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "GBP".to_string(),
                amount: 1000.into(),
            },
        );

        let test_money_gbp = PossiblyNegativeMoney {
            currency: "GBP".to_string(),
            amount: -10,
        };

        let test_money_usd = test_money_gbp
            .convert_to_currency(&bank_registry, "USD".to_string())
            .unwrap();
        let test_money_eur = test_money_usd
            .convert_to_currency(&bank_registry, "EUR".to_string())
            .unwrap();
        let intermediate_test_money_gbp = test_money_usd
            .convert_to_currency(&bank_registry, "GBP".to_string())
            .unwrap();
        let final_test_money_gbp = test_money_eur
            .convert_to_currency(&bank_registry, "GBP".to_string())
            .unwrap();

        assert!(
            test_money_gbp
                .cmp_same_currency(&intermediate_test_money_gbp)
                .unwrap()
                .is_eq()
        );
        assert!(
            test_money_gbp
                .cmp_same_currency(&final_test_money_gbp)
                .unwrap()
                .is_eq()
        );
    }
}
