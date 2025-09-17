use crate::finance::{
    BankRegistry, Money, PersonalFinancePackage, WalletRegistry, WithdrawalError,
};
use crate::person::personal_notes::PersonalNotes;
use crate::person::{Awareness, Boldness, Gender, Morale, Passion, PersonId};

pub struct ThisPerson<'a> {
    pub id: &'a PersonId,
    pub age: &'a u8,
    pub gender: &'a Gender,
    pub passions: &'a [Passion],
    pub morale: &'a Morale,
    pub boldness: &'a Boldness,
    pub awareness: &'a Awareness,
    pub finance: &'a mut PersonalFinancePackage,
    pub notes: &'a mut PersonalNotes,
}

impl<'a> ThisPerson<'a> {
    /// If the wallet doesn't have enough money of specific currency,
    /// the function will try to convert from other currencies contained in the wallet
    /// or attempt to take a credit in a bank.
    /// Returns false if error (must guarantee to have no side effects in that case).
    pub fn ensure_has_money_in_wallet(
        &mut self,
        bank_registry: &BankRegistry,
        wallet_registry: &WalletRegistry,
        money: Money,
    ) -> Result<(), WithdrawalError> {
        assert_ne!(money.amount.unwrap(), 0);

        let mut wallet = self.finance.wallet_mut();

        match wallet.ensure_contains(bank_registry, wallet_registry, money.clone()) {
            Ok(_) => Ok(()),
            Err(err) => {
                drop(wallet);
                let missing = Money {
                    currency: money.currency.clone(),
                    amount: err.missing,
                };

                let borrowed_bank_registry = bank_registry.borrow();
                let bank = borrowed_bank_registry.bank(&missing.currency).unwrap();
                bank.dry_run_withdraw(self.id.clone(), &self.finance.wallet(), missing.amount)?;
                drop(bank);
                self.finance
                    .wallet_mut()
                    .convert_all_into(&bank_registry, missing.currency.clone());

                let mut bank = borrowed_bank_registry.bank_mut(&missing.currency).unwrap();
                bank.withdraw(
                    self.id.clone(),
                    &mut self.finance.wallet_mut(),
                    missing.amount,
                )?;

                assert!(self.finance.wallet().contains(money));
                Ok(())
            }
        }
    }
}
