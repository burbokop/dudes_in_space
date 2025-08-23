use crate::finance::{Currency, MoneyAmount, MoneyRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(flatten)]
    content: BTreeMap<Currency, MoneyAmount>,
}

impl Wallet {
    pub fn new() -> Self {
        todo!()
    }

    pub fn transfer_to(
        &mut self,
        rhs: &mut Wallet,
        m: MoneyRef,
    ) -> Result<(), NotEnoughMoneyInWallet> {
        self.take(m.clone())?;
        rhs.put(m);
        Ok(())
    }

    /// Do not make it public. (Does not preserve the whole amount of money in the system) Should be used only in wallet and bank modules
    pub(crate) fn put(&mut self, m: MoneyRef) {
        *self.content.entry(m.currency).or_default() += m.amount.unwrap();
    }

    /// Do not make it public. (Does not preserve the whole amount of money in the system) Should be used only in wallet and bank modules
    pub(crate) fn amount(&self, c: Currency) -> MoneyAmount {
        match self.content.get(&c) {
            None => 0,
            Some(x) => *x,
        }
    }

    /// Do not make it public. (Does not preserve the whole amount of money in the system) Should be used only in wallet and bank modules
    pub(crate) fn take(&mut self, m: MoneyRef) -> Result<(), NotEnoughMoneyInWallet> {
        let x = self.content.entry(m.currency).or_default();
        if *x >= m.amount.unwrap() {
            Ok(*x -= m.amount.unwrap())
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
