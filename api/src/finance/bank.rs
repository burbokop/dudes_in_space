use crate::finance::{Currency, Money, MoneyAmount, Wallet, WalletId};
use crate::person::PersonId;
use crate::utils::math::{NonNeg, Zero, noneg_float};
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub type Cycle = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccount {
    pub money: MoneyAmount,
    pub growth_rate: NonNeg<Float>,
    pub deadline: Option<Cycle>,
}

impl BankAccount {
    fn new() -> Self {
        Self {
            money: 0,
            growth_rate: noneg_float(0.01),
            deadline: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bank {
    owner: PersonId,
    owner_wallet: WalletId,
    currency: Currency,
    money_stored: MoneyAmount,
    money_created: NonNeg<MoneyAmount>,
    customers: BTreeMap<PersonId, BankAccount>,
    current_cycle: Cycle,
}

impl Bank {
    pub fn new(owner: PersonId, owner_wallet: WalletId, currency: Currency) -> Self {
        Self {
            owner,
            owner_wallet,
            currency,
            money_stored: 0,
            money_created: Zero::zero(),
            customers: BTreeMap::new(),
            current_cycle: 0,
        }
    }

    pub fn owner(&self) -> &PersonId {
        &self.owner
    }

    pub fn owner_wallet(&self) -> &WalletId {
        &self.owner_wallet
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn account(&self, customer: PersonId) -> Option<&BankAccount> {
        self.customers.get(&customer)
    }

    pub(crate) fn customers_count(&self) -> usize {
        self.customers.len()
    }

    pub fn dry_run_withdraw(
        &self,
        customer: PersonId,
        target_wallet: &Wallet,
        amount: NonNeg<MoneyAmount>,
    ) -> Result<(), WithdrawalError> {
        #[derive(Debug)]
        struct Params<'a, 'b> {
            this_bank: &'a Bank,
            customer: PersonId,
            target_wallet: &'b Wallet,
            amount: NonNeg<MoneyAmount>,
        }

        let customers_count = self.customers.len();
        if customer == self.owner {
            Ok(())
        } else {
            let customers_count = match self.customers.get(&customer) {
                None => customers_count + 1,
                Some(account) => {
                    if account.deadline.is_some() {
                        return Err(WithdrawalError::InDebt);
                    }
                    customers_count
                }
            };

            let stored_money_lower_limit =
                -(customers_count as MoneyAmount * self.money_created.unwrap());
            if self.money_stored - amount.unwrap() >= stored_money_lower_limit {
                Ok(())
            } else {
                Err(WithdrawalError::CreditLimitReached {
                    bank_owner: self.owner,
                    requested: NonNeg::new(amount.unwrap() - self.money_stored).unwrap(),
                    limit: NonNeg::new(-stored_money_lower_limit).unwrap(),
                })
            }
        }
    }

    pub fn money_created(&self) -> NonNeg<MoneyAmount> {
        self.money_created
    }

    pub fn can_withdraw(
        &self,
        customer: PersonId,
        target_wallet: &Wallet,
        amount: NonNeg<MoneyAmount>,
    ) -> bool {
        self.dry_run_withdraw(customer, target_wallet, amount)
            .is_ok()
    }

    pub fn withdraw(
        &mut self,
        customer: PersonId,
        target_wallet: &mut Wallet,
        amount: NonNeg<MoneyAmount>,
    ) -> Result<(), WithdrawalError> {
        assert_ne!(amount.unwrap(), 0);

        self.customers.entry(customer).or_insert(BankAccount::new());
        let customers_count = self.customers.len();

        let account = self.customers.get_mut(&customer).unwrap();
        if account.deadline.is_some() {
            return Err(WithdrawalError::InDebt);
        }

        if customer == self.owner {
            account.money -= amount.unwrap();
            self.money_created += amount;
            target_wallet.put(Money {
                currency: self.currency.clone(),
                amount,
            });
            Ok(())
        } else {
            let stored_money_lower_limit =
                -(customers_count as MoneyAmount * self.money_created.unwrap());

            if self.money_stored - amount.unwrap() < stored_money_lower_limit {
                return Err(WithdrawalError::CreditLimitReached {
                    bank_owner: self.owner,
                    requested: NonNeg::new(amount.unwrap() - self.money_stored).unwrap(),
                    limit: NonNeg::new(-stored_money_lower_limit).unwrap(),
                });
            }

            account.money -= amount.unwrap();
            self.money_stored -= amount.unwrap();

            if account.money < 0 {
                account.deadline = Some(
                    self.current_cycle + 2_f64.log(1. + account.growth_rate.unwrap()) as Cycle,
                );
            }
            target_wallet.put(Money {
                currency: self.currency.clone(),
                amount,
            });
            Ok(())
        }
    }

    pub fn deposit(
        &mut self,
        customer: PersonId,
        source_wallet: &mut Wallet,
        amount: NonNeg<MoneyAmount>,
    ) {
        assert_ne!(amount.unwrap(), 0);

        source_wallet
            .take(Money {
                currency: self.currency.clone(),
                amount,
            })
            .unwrap();

        let account = self.customers.entry(customer).or_insert(BankAccount::new());
        account.money += amount.unwrap();
        self.money_stored += amount.unwrap();

        if account.deadline.is_some() && account.money >= 0 {
            account.deadline = None;
        }
    }

    /// Returns amount of money you need to buy `target_amount` from `source_currency_bank`
    /// Note: `target_amount` is in currency of `source_currency_bank`
    /// Note: result is in currency of `self`
    pub fn buy_foreign_currency_price(
        &self,
        target_currency_bank: &Bank,
        target_amount: NonNeg<MoneyAmount>,
    ) -> Result<NonNeg<MoneyAmount>, TargetBankDidNotCreateAnyMoneyError> {
        if target_currency_bank.money_created.unwrap() == 0 {
            return Err(TargetBankDidNotCreateAnyMoneyError {
                currency: target_currency_bank.currency.clone(),
            });
        }

        let source_amount = (target_amount.unwrap() as Float * self.money_created.unwrap() as Float
            / target_currency_bank.money_created.unwrap() as Float)
            as MoneyAmount;

        Ok(NonNeg::new(source_amount).unwrap())
    }

    /// Returns amount of money you get when selling `source_amount` of `self` currency to `target_currency_bank`.
    /// Note: result is in `target_currency_bank` currency
    pub fn sell_this_currency_price(
        &self,
        target_currency_bank: &Bank,
        source_amount: NonNeg<MoneyAmount>,
    ) -> Result<NonNeg<MoneyAmount>, TargetBankDidNotCreateAnyMoneyError> {
        if self.money_created.unwrap() == 0 {
            return Err(TargetBankDidNotCreateAnyMoneyError {
                currency: self.currency.clone(),
            });
        }

        let target_amount = (source_amount.unwrap() as Float
            * target_currency_bank.money_created.unwrap() as Float
            / self.money_created.unwrap() as Float) as MoneyAmount;

        Ok(NonNeg::new(target_amount).unwrap())
    }

    /// Buy `target_amount` currency of `self` by selling corresponding amount of `source_currency_bank` currency
    pub fn buy_currency(
        &mut self,
        bank_owner_wallet: &mut Wallet,
        wallet: &mut Wallet,
        source_currency_bank: &Bank,
        target_amount: NonNeg<MoneyAmount>,
    ) {
        assert_ne!(target_amount.unwrap(), 0);
        assert_ne!(self.money_created, Zero::zero());

        let source_amount = self
            .sell_this_currency_price(source_currency_bank, target_amount)
            .unwrap();

        let money_to_take_from_bank_owner = Money {
            currency: self.currency.clone(),
            amount: target_amount,
        };

        let money_to_take_from_customer = Money {
            currency: source_currency_bank.currency.clone(),
            amount: source_amount,
        };

        {
            let owner_money =
                bank_owner_wallet.amount(money_to_take_from_bank_owner.currency.clone());
            if owner_money < money_to_take_from_bank_owner.amount {
                let delta = money_to_take_from_bank_owner.amount - owner_money;
                self.withdraw(self.owner, bank_owner_wallet, NonNeg::new(delta).unwrap())
                    .unwrap();
            }
        }

        bank_owner_wallet
            .take(money_to_take_from_bank_owner.clone())
            .unwrap();

        wallet.take(money_to_take_from_customer.clone()).unwrap();
        bank_owner_wallet.put(money_to_take_from_customer);
        wallet.put(money_to_take_from_bank_owner);
    }

    pub fn cycle(&mut self) {
        for (customer, account) in &mut self.customers {
            let delta = (account.money as Float * account.growth_rate.unwrap()) as MoneyAmount;

            if let Some(deadline) = account.deadline {
                assert!(deadline < self.current_cycle);
            }

            account.money += delta;
            self.money_stored -= delta;
            self.current_cycle += 1;
        }
    }
}

impl Display for Bank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.currency, self.money_stored, self.money_created
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum WithdrawalError {
    CreditLimitReached {
        bank_owner: PersonId,
        requested: NonNeg<MoneyAmount>,
        limit: NonNeg<MoneyAmount>,
    },
    InDebt,
}

impl Display for WithdrawalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for WithdrawalError {}

#[derive(Debug)]
pub struct TargetBankDidNotCreateAnyMoneyError {
    currency: Currency,
}

impl Display for TargetBankDidNotCreateAnyMoneyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for TargetBankDidNotCreateAnyMoneyError {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::finance::{BankRegistry, WalletRegistry};
    use std::cell::RefCell;
    use std::rc::Rc;

    pub fn create_bank(
        bank_registry: &BankRegistry,
        wallet_registry: &WalletRegistry,
        created_money: Money,
    ) -> (Rc<RefCell<Bank>>, Rc<RefCell<Wallet>>) {
        let owner = PersonId::new_v4();
        let mut owner_wallet = Wallet::new();
        let mut bank = Bank::new(owner, owner_wallet.id().clone(), created_money.currency);

        bank.withdraw(owner, &mut owner_wallet, created_money.amount)
            .unwrap();
        (
            bank_registry.register(bank).unwrap(),
            wallet_registry.register(owner_wallet).unwrap(),
        )
    }

    #[test]
    fn buy_foreign_currency_price_test() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        let (usd_bank, _) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 1000.into(),
            },
        );

        let (eur_bank, _) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "EUR".to_string(),
                amount: 100.into(),
            },
        );

        let usd_bank = usd_bank.borrow_mut();
        let eur_bank = eur_bank.borrow_mut();

        // Returns the amount of USD you need to buy 1 EUR from eur_bank
        assert_eq!(
            usd_bank
                .buy_foreign_currency_price(&eur_bank, 1.into())
                .unwrap(),
            10.into()
        );
    }

    #[test]
    fn sell_this_currency_price_test() {
        let bank_registry = BankRegistry::new();
        let wallet_registry = WalletRegistry::default();

        let (usd_bank, _) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "USD".to_string(),
                amount: 1000.into(),
            },
        );

        let (eur_bank, _) = create_bank(
            &bank_registry,
            &wallet_registry,
            Money {
                currency: "EUR".to_string(),
                amount: 100.into(),
            },
        );

        let usd_bank = usd_bank.borrow_mut();
        let eur_bank = eur_bank.borrow_mut();

        // Returns the amount of EUR you get by selling 10 USD to `eur_bank`
        assert_eq!(
            usd_bank
                .sell_this_currency_price(&eur_bank, 10.into())
                .unwrap(),
            1.into()
        );
    }

    #[test]
    fn dry_run_withdraw_from_virgin_bank_test() {
        let owner = PersonId::new_v4();
        let owner_wallet = Wallet::new();
        let customer = PersonId::new_v4();
        let mut customer_wallet = Wallet::new();
        let currency = "$".into();
        let amount = NonNeg::new(100).unwrap();
        let bank = Bank::new(owner, owner_wallet.id().clone(), currency);

        let result = bank.dry_run_withdraw(customer, &mut customer_wallet, amount);

        assert_eq!(
            result,
            Err(WithdrawalError::CreditLimitReached {
                bank_owner: owner,
                requested: NonNeg::new(100).unwrap(),
                limit: Zero::zero(),
            })
        );
    }

    #[test]
    fn withdraw_from_virgin_bank_test() {
        let owner = PersonId::new_v4();
        let owner_wallet = Wallet::new();

        let customer = PersonId::new_v4();
        let mut customer_wallet = Wallet::new();
        let currency = "$".into();
        let amount = NonNeg::new(100).unwrap();

        let mut bank = Bank::new(owner, owner_wallet.id().clone(), currency);

        let result = bank.withdraw(customer, &mut customer_wallet, amount);

        assert_eq!(
            result,
            Err(WithdrawalError::CreditLimitReached {
                bank_owner: owner,
                requested: NonNeg::new(100).unwrap(),
                limit: Zero::zero(),
            })
        );

        assert_eq!(bank.money_created.unwrap(), 0);
        assert_eq!(bank.money_stored, 0);
        assert_eq!(bank.customers.get(&customer).unwrap().money, 0);
    }

    #[test]
    fn buy_currency_test() {
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

        let mut wallet = Wallet::new();

        let mut usd_bank = usd_bank.borrow_mut();
        let eur_bank = eur_bank.borrow_mut();

        let mut usd_bank_ow = usd_bank_owner_wallet.borrow_mut();
        let mut eur_bank_ow = eur_bank_owner_wallet.borrow_mut();

        usd_bank_ow
            .transfer_to(
                &mut wallet,
                Money {
                    currency: "USD".to_string(),
                    amount: 1000.into(),
                },
            )
            .unwrap();

        eur_bank_ow
            .transfer_to(
                &mut wallet,
                Money {
                    currency: "EUR".to_string(),
                    amount: 1000.into(),
                },
            )
            .unwrap();

        usd_bank.buy_currency(&mut usd_bank_ow, &mut wallet, &eur_bank, 10.into());

        assert_eq!(wallet.amount("USD".to_string()).unwrap(), 1010);
        assert_eq!(wallet.amount("EUR".to_string()).unwrap(), 999);

        assert_eq!(usd_bank_ow.amount("USD".to_string()).unwrap(), 98990);
        assert_eq!(usd_bank_ow.amount("EUR".to_string()).unwrap(), 1);

        assert_eq!(eur_bank_ow.amount("USD".to_string()).unwrap(), 0);
        assert_eq!(eur_bank_ow.amount("EUR".to_string()).unwrap(), 9000);
    }
}
