use serde::{Deserialize, Serialize};
use std::cell::{BorrowError, Ref, RefCell};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::{Rc, Weak};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessToken {
    completed: Weak<RefCell<bool>>,
}

impl ProcessToken {
    pub fn is_completed(&self) -> Result<bool, ProcessTokenExpiredError> {
        match self.completed.upgrade() {
            None => Err(ProcessTokenExpiredError),
            Some(rc) => match rc.try_borrow() {
                Ok(completed) => Ok(*completed),
                Err(_) => Ok(false),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessTokenMut {
    completed: Rc<RefCell<bool>>,
}

impl ProcessTokenMut {
    pub fn new() -> (ProcessToken, Self) {
        let completed = Rc::new(RefCell::new(false));
        (
            ProcessToken {
                completed: Rc::downgrade(&completed),
            },
            Self { completed },
        )
    }

    pub fn mark_completed(&mut self) {
        *self.completed.borrow_mut() = true
    }
}

#[derive(Debug)]
pub struct ProcessTokenExpiredError;

impl Display for ProcessTokenExpiredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ProcessTokenExpiredError {}

struct ProcessTokenContext {
    
}

impl ProcessTokenContext {
    
}