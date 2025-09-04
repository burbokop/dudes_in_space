use crate::finance::{Bank, Currency};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

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
