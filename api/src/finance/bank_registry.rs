use crate::finance::{Bank, Currency};
use std::cell::RefCell;
use std::collections::BTreeMap;
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
    
    pub(crate) fn register(&self, b: Bank) -> Rc<RefCell<Bank>> {
        let currency = b.currency().clone();
        let b = Rc::new(RefCell::new(b));
        self.data.borrow_mut().try_insert(currency, b.clone()).unwrap();
        b
    }
}