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
        target_currency: Currency,
    ) {
        self.content = Money::sum_as(
            bank_registry,
            self.content.iter().map(|(currency, amount)| Money {
                currency: currency.clone(),
                amount: amount.clone(),
            }),
            target_currency,
        )
        .map(|money| BTreeMap::from([(money.currency, money.amount)]))
        .unwrap_or_default();

        todo!(
            "You have a bug in this function. You must do conversion through bank buy currency function"
        )
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
        match self.missing_if_converted(bank_registry, money.clone()) {
            Some(missing) => Err(EnsureContainsError { missing }),
            None => match self.missing(money.clone()) {
                None => Ok(()),
                Some(missing) => {
                    let ok = self.ensure_contains_impl(
                        bank_registry,
                        Money {
                            currency: money.currency,
                            amount: missing,
                        },
                    );
                    assert!(ok);
                    Ok(())
                }
            },
        }
    }

    fn ensure_contains_impl(&mut self, bank_registry: &BankRegistry, money: Money) -> bool {
        let mut missing = money.clone();

        for (currency, amount) in &self.content {
            if currency == &money.currency {
                missing.amount.sub_assign(*amount).unwrap();
                break;
            }
        }
        self.content.remove(&money.currency);



        self.content.retain(|currency, amount| {
            let current = Money {
                currency: currency.clone(),
                amount: amount.clone(),
            };

            let current = current.convert_to_currency(bank_registry, missing.currency.clone());

            let min = current.clone().min(bank_registry, missing.clone());

            missing.sub_assign_same_currency(min.clone()).unwrap();

            todo!("Get the bank from the bank registry and buy the currency from it");

            min.amount != current.amount
        });

        true
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
