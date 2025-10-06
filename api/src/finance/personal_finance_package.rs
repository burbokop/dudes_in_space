use crate::finance::{Bank, BankRegistry, Currency, MoneyAmount, Wallet, WalletRegistry};
use crate::utils::math::{NonNeg, Zero};
use serde::de::{DeserializeSeed, Error};
use serde::{Deserialize, Deserializer, Serialize};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Debug, Serialize)]
pub struct PersonalFinancePackage {
    #[serde(with = "crate::utils::tagged_option")]
    bank: Option<Rc<RefCell<Bank>>>,
    wallet: Rc<RefCell<Wallet>>,
}

impl Default for PersonalFinancePackage {
    fn default() -> Self {
        PersonalFinancePackage {
            bank: None,
            wallet: Rc::new(RefCell::new(Wallet::new())),
        }
    }
}

impl PersonalFinancePackage {
    pub fn bank(&self) -> Option<Ref<'_, Bank>> {
        self.bank.as_ref().map(|x| x.borrow())
    }

    pub fn wallet<'a>(&'a self) -> Ref<'a, Wallet> {
        self.wallet.borrow()
    }

    pub fn wallet_mut<'a>(&'a self) -> RefMut<'a, Wallet> {
        self.wallet.borrow_mut()
    }

    pub fn preferred_currency(&self, bank_registry: &BankRegistry) -> Option<Currency> {
        match self.wallet.borrow().most_worth_currency(bank_registry) {
            None => self.bank().map(|b| b.currency().clone()),
            Some(c) => Some(c.currency),
        }
    }

    pub fn preferred_currency_or_create(
        &mut self,
        bank_registry: &BankRegistry,
        new_bank: Bank,
    ) -> Currency {
        if let Some(m) = self.wallet.borrow().most_worth_currency(bank_registry) {
            return m.currency;
        }

        if let Some(c) = self.bank().map(|b| b.currency().clone()) {
            return c;
        }

        {
            let bank_registry = bank_registry.borrow();
            if let Some(c) = bank_registry.bank_with_most_customers() {
                return c.currency().clone();
            }
        }

        let currency = new_bank.currency().clone();
        self.bank = Some(bank_registry.register(new_bank).unwrap());
        currency
    }

    pub fn increase_credit_limit_or_create(
        &mut self,
        new_limit: NonNeg<MoneyAmount>,
        mut new_bank: Bank,
    ) {
        assert_ne!(new_limit.into_inner(), 0);
        match &self.bank {
            None => {
                match NonNeg::new(new_limit - new_bank.money_created()) {
                    Ok(missing) => {
                        new_bank
                            .withdraw(*new_bank.owner(), &mut self.wallet.borrow_mut(), missing)
                            .unwrap();
                    }
                    _ => {}
                }
                self.bank = Some(Rc::new(RefCell::new(new_bank)))
            }
            Some(bank) => {
                let mut bank = bank.borrow_mut();
                match NonNeg::new(new_limit - bank.money_created()) {
                    Ok(missing) if missing != NonNeg::zero() => {
                        let owner = bank.owner().clone();
                        bank.withdraw(owner, &mut self.wallet.borrow_mut(), missing)
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct PersonalFinancePackageSeed<'a, 'b> {
    bank_registry: &'a BankRegistry,
    wallet_registry: &'b WalletRegistry,
}

impl<'a, 'b> PersonalFinancePackageSeed<'a, 'b> {
    pub(crate) fn new(
        bank_registry: &'a BankRegistry,

        wallet_registry: &'b WalletRegistry,
    ) -> Self {
        Self {
            bank_registry,
            wallet_registry,
        }
    }
}

impl<'de, 'a, 'b> DeserializeSeed<'de> for PersonalFinancePackageSeed<'a, 'b> {
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
            bank: bank
                .map(|bank| self.bank_registry.register(bank))
                .map_or(Ok(None), |v| v.map(Some))
                .map_err(Error::custom)?,
            wallet: self
                .wallet_registry
                .register(wallet)
                .map_err(Error::custom)?,
        })
    }
}
