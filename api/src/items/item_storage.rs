use crate::{DuplicateItemError, InputRecipe, Item, ItemCount, ItemId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemStorage {
    content: BTreeMap<ItemId, ItemCount>,
}

impl FromIterator<Item> for ItemStorage {
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        todo!()
    }
}

impl TryFrom<Vec<Item>> for ItemStorage {
    type Error = DuplicateItemError;

    fn try_from(value: Vec<Item>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl ItemStorage {
    pub fn new() -> Self {
        Default::default()
    }

    /// returns the rest that did not fit inside storage space
    pub fn add_item(&mut self, item: Item) -> Item {
        todo!()
    }

    /// returns true if item was added, false if not due to full storage
    pub fn try_add_item(&mut self, item: Item) -> bool {
        todo!()
    }

    /// remove as many items as possible
    pub fn remove_item(&mut self, item_id: ItemId, count: ItemCount) -> Item {
        todo!()
    }

    /// returns true if item was removed, false if not due to not enough item count in storage
    pub fn try_remove_item(&mut self, item: Item) -> bool {
        todo!()
    }

    pub fn count(&self, item_id: ItemId) -> ItemCount {
        todo!()
    }

    pub fn contains(&self, item: &Item) -> bool {
        match self.content.get(item.id()) {
            None => false,
            Some(count) => *count >= item.count(),
        }
    }

    pub fn contains_for_input(&self, input: InputRecipe) -> bool {
        input.into_iter().all(|item| self.contains(&item))
    }

    pub fn try_consume(&mut self, input: InputRecipe) -> bool {
        let ok = self.contains_for_input(input.clone());
        if (!ok) {
            return false;
        }

        for item in input.into_iter() {
            let c = self.content.get_mut(item.id()).unwrap();
            *c -= item.count();
            if *c == 0 {
                self.content.remove(item.id());
            }
        }

        true
    }
}
