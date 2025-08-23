use crate::finance::{Currency, MoneyAmount, MoneyRef, Wallet};
use crate::person::PersonId;
use crate::utils::math::{NoNeg, noneg_float};
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Cycle = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccount {
    pub money: MoneyAmount,
    pub growth_rate: NoNeg<Float>,
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
    currency: Currency,
    money_stored: MoneyAmount,
    money_created: MoneyAmount,
    customers: BTreeMap<PersonId, BankAccount>,
    current_cycle: Cycle,
}

impl Bank {
    pub fn new(owner: PersonId, currency: Currency) -> Self {
        Self {
            owner,
            currency,
            money_stored: 0,
            money_created: 0,
            customers: BTreeMap::new(),
            current_cycle: 0,
        }
    }
    
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn account(&self, customer: PersonId) -> Option<&BankAccount> {
        self.customers.get(&customer)
    }

    pub fn withdraw(
        &mut self,
        customer: PersonId,
        target_wallet: &mut Wallet,
        amount: NoNeg<MoneyAmount>,
    ) {
        assert_ne!(amount.unwrap(), 0);

        self.customers.entry(customer).or_insert(BankAccount::new());
        let customers_count = self.customers.len();

        let account = self.customers.get_mut(&customer).unwrap();
        assert!(account.deadline.is_none());

        if customer == self.owner {
            account.money -= amount.unwrap();
            self.money_created += amount.unwrap();
            target_wallet.put(MoneyRef {
                currency: self.currency.clone(),
                amount,
            })
        } else {
            account.money -= amount.unwrap();

            let stored_money_lower_limit = -(customers_count as MoneyAmount * self.money_created);

            assert!(self.money_stored - amount.unwrap() >= stored_money_lower_limit);

            self.money_stored -= amount.unwrap();

            if account.money < 0 {
                account.deadline = Some(
                    self.current_cycle + 2_f64.log(1. + account.growth_rate.unwrap()) as Cycle,
                );
            }
            target_wallet.put(MoneyRef {
                currency: self.currency.clone(),
                amount,
            })
        }
    }

    pub fn deposit(
        &mut self,
        customer: PersonId,
        source_wallet: &mut Wallet,
        amount: NoNeg<MoneyAmount>,
    ) {
        assert_ne!(amount.unwrap(), 0);

        source_wallet
            .take(MoneyRef {
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

    pub fn currency_price(
        &self,
        source_currency_bank: &Bank,
        target_amount: NoNeg<MoneyAmount>,
    ) -> NoNeg<MoneyAmount> {
        let source_amount = (target_amount.unwrap() as Float
            * source_currency_bank.money_created as Float
            / self.money_created as Float) as MoneyAmount;
        NoNeg::wrap(source_amount).unwrap()
    }

    pub fn buy_currency(
        &mut self,
        bank_owner_wallet: &mut Wallet,
        wallet: &mut Wallet,
        source_currency_bank: &Bank,
        target_amount: NoNeg<MoneyAmount>,
    ) {
        assert_ne!(target_amount.unwrap(), 0);
        assert!(self.money_created > 0);

        let source_amount = (target_amount.unwrap() as Float
            * source_currency_bank.money_created as Float
            / self.money_created as Float) as MoneyAmount;

        let money_to_take_from_bank_owner = MoneyRef {
            currency: self.currency.clone(),
            amount: target_amount,
        };

        let money_to_take_from_customer = MoneyRef {
            currency: self.currency.clone(),
            amount: target_amount,
        };

        {
            let owner_money =
                bank_owner_wallet.amount(money_to_take_from_bank_owner.currency.clone());
            if owner_money < money_to_take_from_bank_owner.amount.unwrap() {
                let delta = money_to_take_from_bank_owner.amount.unwrap() - owner_money;
                self.withdraw(self.owner, bank_owner_wallet, NoNeg::wrap(delta).unwrap());
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
