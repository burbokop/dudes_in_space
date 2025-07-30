use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

pub type ItemId = String;
pub type ItemCount = u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub(crate) id: ItemId,
    pub(crate) count: ItemCount,
}

impl Item {
    pub fn new(id: ItemId, count: ItemCount) -> Self {
        Self { id, count }
    }

    pub fn id(&self) -> &ItemId {
        &self.id
    }
    pub fn count(&self) -> ItemCount {
        self.count
    }
}

#[derive(Debug)]
pub struct DuplicateItemError;

impl Display for DuplicateItemError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for DuplicateItemError {}
