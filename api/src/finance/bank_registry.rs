use crate::finance::{Bank, Currency};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct BankRegistry {
    data: RefCell<BTreeMap<Currency, Rc<RefCell<Bank>>>>,
}

pub struct BorrowedBankRegistry<'a> {
    data: Ref<'a, BTreeMap<Currency, Rc<RefCell<Bank>>>>,
}

impl<'a> BorrowedBankRegistry<'a> {
    pub fn bank<'b>(&'b self, currency: &Currency) -> Option<Ref<'b, Bank>>
    where
        'b: 'a,
    {
        self.data.get(currency).map(|x| x.borrow())
    }

    pub fn bank_mut<'b>(&'b self, currency: &Currency) -> Option<RefMut<'b, Bank>>
    where
        'b: 'a,
    {
        self.data.get(currency).map(|x| x.borrow_mut())
    }

    pub(crate) fn bank_with_most_customers<'b>(&'b self) -> Option<Ref<'b, Bank>>
    where
        'b: 'a,
    {
        self.data
            .iter()
            .max_by(|(_, bank0), (_, bank1)| {
                let bank0_cc = bank0.borrow().customers_count();
                let bank1_cc = bank1.borrow().customers_count();
                bank1_cc.cmp(&bank0_cc)
            })
            .map(|(_, x)| x.borrow())
    }
}

impl BankRegistry {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }

    pub fn contains_currency(&self, currency: &Currency) -> bool {
        self.data.borrow().contains_key(currency)
    }

    pub fn borrow<'a>(&'a self) -> BorrowedBankRegistry<'a> {
        BorrowedBankRegistry {
            data: self.data.borrow(),
        }
    }

    pub(crate) fn register(
        &self,
        b: Bank,
    ) -> Result<Rc<RefCell<Bank>>, CurrencyAlreadyExistsError> {
        let currency = b.currency().clone();
        let b = Rc::new(RefCell::new(b));
        self.data
            .borrow_mut()
            .try_insert(currency, b.clone())
            .map_err(|_| CurrencyAlreadyExistsError)?;
        Ok(b)
    }
}

#[derive(Debug)]
pub(crate) struct CurrencyAlreadyExistsError;

impl Display for CurrencyAlreadyExistsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CurrencyAlreadyExistsError {}
