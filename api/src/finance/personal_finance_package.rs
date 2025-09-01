use crate::finance::{Bank, BankRegistry, Currency, Wallet};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

#[derive(Default, Debug, Serialize)]
pub struct PersonalFinancePackage {
    #[serde(with = "crate::utils::tagged_option")]
    bank: Option<Rc<RefCell<Bank>>>,
    wallet: Wallet,
}

impl PersonalFinancePackage {
    pub fn bank(&self) -> Option<Ref<'_, Bank>> {
        self.bank.as_ref().map(|x| x.borrow())
    }
    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    pub fn preferred_currency(&self,bank_registry: & BankRegistry,) -> Option<Currency> {
        match self.wallet.most_worth_currency(bank_registry) {
            None => self.bank().map(|b|b.currency().clone()),
            Some(c) => Some(c.currency),
        }
    }

    pub fn preferred_currency_or_create(&self, bank_registry: &BankRegistry, new_currency_name: String) -> Currency {
        match self.wallet.most_worth_currency(bank_registry) {
            None => match self.bank().map(|b|b.currency().clone()) {
                None => todo!(),
                Some(c) => c,
            },
            Some(c) => c.currency,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PersonalFinancePackageSeed<'b> {
    bank_registry: &'b BankRegistry,
}

impl<'b> PersonalFinancePackageSeed<'b> {
    pub(crate) fn new(bank_registry: &'b BankRegistry) -> Self {
        Self { bank_registry }
    }
}

impl<'de, 'b> DeserializeSeed<'de> for PersonalFinancePackageSeed<'b> {
    type Value = PersonalFinancePackage;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Impl {
            #[serde(with = "crate::utils::tagged_option")]
            bank: Option<Bank>,
            wallet: Wallet,
        }

        let Impl { bank, wallet } = Impl::deserialize(deserializer)?;

        Ok(Self::Value {
            bank: bank.map(|bank| self.bank_registry.register(bank)),
            wallet,
        })
    }
}
