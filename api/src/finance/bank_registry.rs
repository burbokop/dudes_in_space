use crate::finance::{Bank, Currency};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub struct BankRegistry {
    data: RefCell<BTreeMap<Currency, Rc<RefCell<Bank>>>>,
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
