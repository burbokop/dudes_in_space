use crate::person::PersonId;
use crate::utils::math::{NoNeg, noneg_float};
use crate::utils::utils::Float;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub type Currency = String;
pub type MoneyAmount = i64;
pub type Cycle = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Money {
    currency: Currency,
    amount: NoNeg<MoneyAmount>,
}

impl Default for Money {
    fn default() -> Self {
        Self {
            amount: NoNeg::wrap(0).unwrap(),
            currency: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyRef {
    pub currency: Currency,
    pub amount: NoNeg<MoneyAmount>,
}

impl Ord for MoneyRef {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}

impl Money {
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn amount(&self) -> NoNeg<MoneyAmount> {
        self.amount
    }
}

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

    pub fn account(&self, customer: PersonId) -> Option<&BankAccount> {
        self.customers.get(&customer)
    }

    pub fn withdraw(&mut self, customer: PersonId, amount: NoNeg<MoneyAmount>) -> Money {
        assert_ne!(amount.unwrap(), 0);

        self.customers.entry(customer).or_insert(BankAccount::new());
        let customers_count = self.customers.len();

        let account = self.customers.get_mut(&customer).unwrap();
        assert!(account.deadline.is_none());

        if customer == self.owner {
            account.money -= amount.unwrap();
            self.money_created += amount.unwrap();
            Money {
                currency: self.currency.clone(),
                amount,
            }
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

            Money {
                currency: self.currency.clone(),
                amount,
            }
        }
    }

    pub fn deposit(&mut self, customer: PersonId, money: Money) {
        assert_eq!(self.currency, money.currency);
        assert_ne!(money.amount.unwrap(), 0);

        let account = self.customers.entry(customer).or_insert(BankAccount::new());
        account.money += money.amount.unwrap();
        self.money_stored += money.amount.unwrap();

        if account.deadline.is_some() && account.money >= 0 {
            account.deadline = None;
        }
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
