use crate::utils::math::Complex;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use std::cell::{BorrowError, Ref, RefCell};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::rc::{Rc, Weak};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessToken {
    #[serde(skip)]
    completed: Option<Weak<RefCell<bool>>>,
    id: Uuid,
}

impl ProcessToken {
    pub fn is_completed(
        &mut self,
        context: &ProcessTokenContext,
    ) -> Result<bool, ProcessTokenExpiredError> {
        match &self.completed {
            None => {
                let data = context.data.borrow();
                
                let weak = data.get(&self.id).ok_or(ProcessTokenExpiredError)?;
                
                self.completed = Some(weak.clone());
                match weak.upgrade() {
                    None => {
                        context.data.borrow_mut().remove(&self.id);
                        Err(ProcessTokenExpiredError)
                    },
                    Some(rc) => Ok(*rc.borrow()),
                }
            }
            Some(completed) => match completed.upgrade() {
                None => Err(ProcessTokenExpiredError),
                Some(rc) => Ok(*rc.borrow()),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProcessTokenMut {
    completed: Rc<RefCell<bool>>,
    id: Uuid,
}

#[derive(Clone)]
pub struct ProcessTokenMutSeed<'context> {
    context: &'context ProcessTokenContext,
}

impl<'context> ProcessTokenMutSeed<'context> {
    pub fn new(context: &'context ProcessTokenContext) -> Self {
        Self { context }
    }
}

impl<'de, 'context> DeserializeSeed<'de> for ProcessTokenMutSeed<'context> {
    type Value = ProcessTokenMut;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Impl {
            completed: bool,
            id: Uuid,
        }

        let Impl { completed, id } = Impl::deserialize(deserializer)?;
        Ok(self.context.register(completed, id))
    }
}

impl ProcessTokenMut {
    pub fn new() -> (ProcessToken, Self) {
        let completed = Rc::new(RefCell::new(false));
        let id = Uuid::new_v4();
        (
            ProcessToken {
                completed: Some(Rc::downgrade(&completed)),
                id,
            },
            Self { completed, id },
        )
    }

    pub fn mark_completed(&mut self, context: &ProcessTokenContext) {
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

pub struct ProcessTokenContext {
    data: RefCell<BTreeMap<Uuid, Weak<RefCell<bool>>>>,
}

impl ProcessTokenContext {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(BTreeMap::new()),
        }
    }

    fn register(&self, completed: bool, id: Uuid) -> ProcessTokenMut {
        let completed = Rc::new(RefCell::new(completed));
        self.data
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&completed))
            .unwrap();
        ProcessTokenMut { completed, id }
    }
}
