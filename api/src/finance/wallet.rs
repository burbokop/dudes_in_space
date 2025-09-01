use crate::finance::{BankRegistry, Currency, Money, MoneyAmount};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::utils::math::{NonNeg, Zero};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(flatten)]
    content: BTreeMap<Currency, NonNeg<MoneyAmount>>,
}

impl Wallet {
    pub fn most_worth_currency(&self, bank_registry: &BankRegistry) -> Option<Money> {
        self.content.iter().map(|(currency, amount)| {
            Money { currency: currency.clone(), amount: *amount }
        }).max_by(|a,b| a.cmp(b, bank_registry) )
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
}

#[derive(Debug)]
pub struct NotEnoughMoneyInWallet;

impl Display for NotEnoughMoneyInWallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for NotEnoughMoneyInWallet {}
