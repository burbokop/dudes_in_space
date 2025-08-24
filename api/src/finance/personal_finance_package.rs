use crate::finance::{Bank, BankRegistry, Wallet};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Debug, Serialize)]
pub struct PersonalFinancePackage {
    #[serde(with = "crate::utils::tagged_option")]
    bank: Option<Rc<RefCell<Bank>>>,
    wallet: Wallet,
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
