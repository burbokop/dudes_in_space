use crate::finance::{BankRegistry, Currency, Money, MoneyAmount};
use crate::utils::math::{NonNeg, Zero};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(flatten)]
    content: BTreeMap<Currency, NonNeg<MoneyAmount>>,
}

impl Wallet {
    pub fn most_worth_currency(&self, bank_registry: &BankRegistry) -> Option<Money> {
        self.content
            .iter()
            .map(|(currency, amount)| Money {
                currency: currency.clone(),
                amount: *amount,
            })
            .max_by(|a, b| a.cmp(bank_registry, b))
    }

    pub fn sum(&self, bank_registry: &BankRegistry) -> Option<Money> {
        let currency = self.most_worth_currency(bank_registry)?.currency;
        Money::sum_as(
            bank_registry,
            self.content.iter().map(|(currency, amount)| Money {
                currency: currency.clone(),
                amount: amount.clone(),
            }),
            currency,
        )
    }

    pub fn transfer_to(
        &mut self,
        rhs: &mut Wallet,
        m: Money,
    ) -> Result<(), NotEnoughMoneyInWallet> {
        self.take(m.clone())?;
        rhs.put(m);
        Ok(())
    }

    /// Do not make it public. (Does not preserve the whole amount of money in the system) Should be used only in wallet and bank modules
    pub(crate) fn put(&mut self, m: Money) {
        *self.content.entry(m.currency).or_default() += m.amount;
    }

    pub(crate) fn amount(&self, c: Currency) -> NonNeg<MoneyAmount> {
        match self.content.get(&c) {
            None => Zero::zero(),
            Some(x) => *x,
        }
    }

    /// Do not make it public. (Does not preserve the whole amount of money in the system) Should be used only in wallet and bank modules
    pub(crate) fn take(&mut self, m: Money) -> Result<(), NotEnoughMoneyInWallet> {
        let x = self.content.entry(m.currency).or_default();
        if *x >= m.amount {
            Ok(x.sub_assign(m.amount).unwrap())
        } else {
            Err(NotEnoughMoneyInWallet)
        }
    }

    /// Checks whether the wallet contains money of exact currency
    pub(crate) fn contains(&self, money: Money) -> bool {
        match self.content.get(&money.currency) {
            None => false,
            Some(amount) => money.amount <= *amount,
        }
    }

    pub(crate) fn convert_all_into(&mut self, currency: Currency) {}

    /// Checks whether the wallet contains money if converted to this currency.
    /// Returns `None` if it has enough and `MoneyAmount` if missing.
    pub(crate) fn missing_if_converted(
        &self,
        bank_registry: &BankRegistry,
        money: Money,
    ) -> Option<NonNeg<MoneyAmount>> {
        if self.contains(money.clone()) {
            return None;
        }

        match Money::sum_as(
            bank_registry,
            self.content.iter().map(|(currency, amount)| Money {
                currency: currency.clone(),
                amount: amount.clone(),
            }),
            money.currency.clone(),
        ) {
            None => Some(money.amount),
            Some(sum) => {
                if money.amount > sum.amount {
                    Some(NonNeg::new(money.amount - sum.amount).unwrap())
                } else {
                    None
                }
            }
        }
    }

    /// If the wallet doesn't have enough money of specific currency, the function will try to convert from other currencies contained in the wallet
    /// Returns false if error (must guarantee to have no side effects in that case)
    pub(crate) fn ensure_contains(
        &mut self,
        bank_registry: &BankRegistry,
        money: Money,
    ) -> Result<(), EnsureContainsError> {
        match self.missing_if_converted(bank_registry, money) {
            Some(missing) => Err(EnsureContainsError { missing }),
            None => {
                todo!()
                // Ok()
            }
        }
    }
}

#[derive(Debug)]
pub struct EnsureContainsError {
    pub missing: NonNeg<MoneyAmount>,
}

impl Display for EnsureContainsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for EnsureContainsError {}

#[derive(Debug)]
pub struct NotEnoughMoneyInWallet;

impl Display for NotEnoughMoneyInWallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for NotEnoughMoneyInWallet {}

impl Display for Wallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, (currency, count)) in self.content.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}:{}", currency, count)?;
        }
        write!(f, "]")
    }
}
