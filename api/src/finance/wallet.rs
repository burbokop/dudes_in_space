use crate::finance::{BankRegistry, Currency, Money, MoneyAmount};
use crate::utils::math::{NonNeg, Zero};
use crate::utils::non_nil_uuid::NonNilUuid;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::{Rc, Weak};

pub type WalletId = NonNilUuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    content: BTreeMap<Currency, NonNeg<MoneyAmount>>,
    id: WalletId,
}

impl Wallet {
    pub fn new() -> Self {
        Self {
            content: Default::default(),
            id: WalletId::new_v4(),
        }
    }

    pub fn id(&self) -> &WalletId {
        &self.id
    }

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
        .unwrap()
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

    pub(crate) fn convert_all_into(
        &mut self,
        bank_registry: &BankRegistry,
        wallet_registry: &WalletRegistry,
        target_currency: Currency,
    ) {
        for (currency, amount) in self.content.clone() {
            if currency == target_currency {
                continue;
            }

            let current_money = Money {
                currency: currency.clone(),
                amount: amount.clone(),
            };

            let bank_registry = bank_registry.borrow();

            let mut target_bank = bank_registry.bank_mut(&target_currency).unwrap();
            let mut current_bank = bank_registry.bank_mut(&current_money.currency).unwrap();

            let target_bank_owner_wallet =
                wallet_registry.get(&target_bank.owner_wallet()).unwrap();
            let target_bank_owner_wallet = target_bank_owner_wallet.upgrade().unwrap();
            let mut target_bank_owner_wallet = target_bank_owner_wallet.borrow_mut();

            let current_bank_owner_wallet =
                wallet_registry.get(&current_bank.owner_wallet()).unwrap();
            let current_bank_owner_wallet = current_bank_owner_wallet.upgrade().unwrap();
            let current_bank_owner_wallet = current_bank_owner_wallet.borrow_mut();

            let target_amount = current_bank
                .sell_this_currency_price(&target_bank, amount)
                .unwrap();

            target_bank.buy_currency(
                &mut target_bank_owner_wallet,
                self,
                &mut current_bank,
                target_amount,
            );
        }
    }

    pub(crate) fn missing(&mut self, money: Money) -> Option<NonNeg<MoneyAmount>> {
        let amount_containing = self
            .content
            .get(&money.currency)
            .cloned()
            .unwrap_or(Zero::zero());
        NonNeg::new(money.amount - amount_containing).ok()
    }

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
        )
        .unwrap()
        {
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
        wallet_registry: &WalletRegistry,
        target_money: Money,
    ) -> Result<(), EnsureContainsError> {
        match self.missing_if_converted(bank_registry, target_money.clone()) {
            Some(missing) => Err(EnsureContainsError { missing }),
            None => match self.missing(target_money.clone()) {
                None => Ok(()),
                Some(missing) => {
                    let ok =
                        self.ensure_contains_impl(bank_registry, wallet_registry, target_money);
                    assert!(ok);
                    Ok(())
                }
            },
        }
    }

    fn ensure_contains_impl(
        &mut self,
        bank_registry: &BankRegistry,
        wallet_registry: &WalletRegistry,
        target_money: Money,
    ) -> bool {
        'l: loop {
            for (currency, amount) in self.content.clone() {
                if *currency == target_money.currency {
                    continue;
                }

                let current_amount_of_target_currency = self
                    .content
                    .get(&target_money.currency)
                    .cloned()
                    .unwrap_or(Zero::zero());

                let delta =
                    NonNeg::new(target_money.amount - current_amount_of_target_currency).unwrap();

                if delta == Zero::zero() {
                    self.content.retain(|_, a| *a != Zero::zero());
                    break 'l true;
                }

                if self.content.len() <= 1 {
                    break 'l false;
                }

                let delta_money = Money {
                    currency: target_money.currency.clone(),
                    amount: delta.clone(),
                };

                let current_money = Money {
                    currency: currency.clone(),
                    amount: amount.clone(),
                };

                let current_money_in_target_currency = current_money
                    .convert_to_currency(bank_registry, target_money.currency.clone())
                    .unwrap();

                let min = current_money_in_target_currency
                    .clone()
                    .min_same_currency(delta_money.clone())
                    .unwrap();

                let bank_registry = bank_registry.borrow();

                let mut target_bank = bank_registry.bank_mut(&target_money.currency).unwrap();
                let mut current_bank = bank_registry.bank_mut(&current_money.currency).unwrap();

                let target_bank_owner_wallet =
                    wallet_registry.get(&target_bank.owner_wallet()).unwrap();
                let target_bank_owner_wallet = target_bank_owner_wallet.upgrade().unwrap();
                let mut target_bank_owner_wallet = target_bank_owner_wallet.borrow_mut();

                target_bank.buy_currency(
                    &mut target_bank_owner_wallet,
                    self,
                    &mut current_bank,
                    min.amount,
                );

                if self.content.get(&target_money.currency).unwrap().clone() >= target_money.amount
                {
                    break;
                }
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

#[derive(Debug, Clone)]
pub struct WeakWallet {
    data: Weak<RefCell<Wallet>>,
}

impl WeakWallet {
    pub fn upgrade(&self) -> Option<Rc<RefCell<Wallet>>> {
        self.data.upgrade()
    }

    // pub fn as_ref<'a>(&'a self) -> Ref<'a, Wallet> {
    //     todo!()
    // }
    //
    // pub fn as_mut<'a>(&'a self) -> RefMut<'a, Wallet> {
    //     self.data.upgrade().unwrap().borrow_mut()
    // }
}

#[derive(Debug)]
pub struct WalletRegistry {
    data: RefCell<BTreeMap<WalletId, WeakWallet>>,
}

impl Default for WalletRegistry {
    fn default() -> Self {
        Self {
            data: RefCell::new(BTreeMap::new()),
        }
    }
}

impl WalletRegistry {
    pub(crate) fn register(
        &self,
        b: Wallet,
    ) -> Result<Rc<RefCell<Wallet>>, PersonAlreadyExistsError> {
        let id = b.id().clone();
        let b = Rc::new(RefCell::new(b));
        self.data
            .borrow_mut()
            .try_insert(
                id,
                WeakWallet {
                    data: Rc::downgrade(&b),
                },
            )
            .map_err(|_| PersonAlreadyExistsError)?;
        Ok(b)
    }

    pub(crate) fn get(&self, wallet_id: &WalletId) -> Option<WeakWallet> {
        self.data.borrow().get(wallet_id).cloned()
    }
}

#[derive(Debug)]
pub struct PersonAlreadyExistsError;

impl Display for PersonAlreadyExistsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PersonAlreadyExistsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::tests::create_bank;

    #[test]
    fn convert_all_into_test() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        let (usd_bank, usd_bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 100000.into(),
            },
        );

        let (eur_bank, eur_bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "EUR".to_string(),
                amount: 10000.into(),
            },
        );

        let (gbp_bank, gbp_bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "GBP".to_string(),
                amount: 1000.into(),
            },
        );

        let mut wallet = Wallet::new();

        {
            let mut usd_bank_owner_wallet = usd_bank_owner_wallet.borrow_mut();
            let mut eur_bank_owner_wallet = eur_bank_owner_wallet.borrow_mut();
            let mut gbp_bank_owner_wallet = gbp_bank_owner_wallet.borrow_mut();

            usd_bank_owner_wallet
                .transfer_to(
                    &mut wallet,
                    Money {
                        currency: "USD".to_string(),
                        amount: 1000.into(),
                    },
                )
                .unwrap();

            eur_bank_owner_wallet
                .transfer_to(
                    &mut wallet,
                    Money {
                        currency: "EUR".to_string(),
                        amount: 1000.into(),
                    },
                )
                .unwrap();

            gbp_bank_owner_wallet
                .transfer_to(
                    &mut wallet,
                    Money {
                        currency: "GBP".to_string(),
                        amount: 1000.into(),
                    },
                )
                .unwrap();
        }

        wallet.convert_all_into(&bank_registry, &wallet_registry, "USD".to_string());

        assert_eq!(wallet.amount("USD".to_string()).unwrap(), 111000);
        assert_eq!(wallet.amount("EUR".to_string()).unwrap(), 0);
        assert_eq!(wallet.amount("GBP".to_string()).unwrap(), 0);
    }

    #[test]
    fn transfer_to_test() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        let (bank, bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 100000.into(),
            },
        );

        let mut bank_owner_wallet = bank_owner_wallet.borrow_mut();
        let mut wallet = Wallet::new();

        bank_owner_wallet
            .transfer_to(
                &mut wallet,
                Money {
                    currency: "USD".to_string(),
                    amount: 1000.into(),
                },
            )
            .unwrap();

        assert_eq!(bank_owner_wallet.amount("USD".to_string()).unwrap(), 99000);
        assert_eq!(wallet.amount("USD".to_string()).unwrap(), 1000);
    }

    #[test]
    fn ensure_contains_impl() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        let (usd_bank, usd_bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 100000.into(),
            },
        );

        let (eur_bank, eur_bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "EUR".to_string(),
                amount: 10000.into(),
            },
        );

        let (gbp_bank, gbp_bank_owner_wallet) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "GBP".to_string(),
                amount: 1000.into(),
            },
        );

        let mut wallet = Wallet::new();

        {
            let mut usd_bank_owner_wallet = usd_bank_owner_wallet.borrow_mut();
            let mut eur_bank_owner_wallet = eur_bank_owner_wallet.borrow_mut();
            let mut gbp_bank_owner_wallet = gbp_bank_owner_wallet.borrow_mut();

            usd_bank_owner_wallet
                .transfer_to(
                    &mut wallet,
                    Money {
                        currency: "USD".to_string(),
                        amount: 1000.into(),
                    },
                )
                .unwrap();

            eur_bank_owner_wallet
                .transfer_to(
                    &mut wallet,
                    Money {
                        currency: "EUR".to_string(),
                        amount: 1000.into(),
                    },
                )
                .unwrap();

            gbp_bank_owner_wallet
                .transfer_to(
                    &mut wallet,
                    Money {
                        currency: "GBP".to_string(),
                        amount: 1000.into(),
                    },
                )
                .unwrap();
        }

        wallet.ensure_contains_impl(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 20000.into(),
            },
        );

        assert_eq!(wallet.amount("USD".to_string()).unwrap(), 20000);
        assert_eq!(wallet.amount("EUR".to_string()).unwrap(), 0);
        assert_eq!(wallet.amount("GBP".to_string()).unwrap(), 910);
    }
}
