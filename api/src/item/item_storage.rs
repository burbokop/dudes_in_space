use crate::item::{DuplicateItemError, Item, ItemCount, ItemId};
use crate::recipe::InputRecipe;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemStorage {
    content: BTreeMap<ItemId, ItemCount>,
    capacity: ItemCount,
    #[serde(skip)]
    total_count: ItemCount,
}

impl<'de> Deserialize<'de> for ItemStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Impl {
            content: BTreeMap<ItemId, ItemCount>,
            capacity: ItemCount,
        }

        let Impl { content, capacity } = Impl::deserialize(deserializer)?;
        let total_count = Self::eval_total_count(&content);
        Ok(Self {
            content,
            capacity,
            total_count,
        })
    }
}

impl FromIterator<Item> for ItemStorage {
    fn from_iter<T: IntoIterator<Item = Item>>(_iter: T) -> Self {
        todo!()
    }
}

impl TryFrom<Vec<Item>> for ItemStorage {
    type Error = DuplicateItemError;

    fn try_from(value: Vec<Item>) -> Result<Self, Self::Error> {
        todo!()
        // let mut result = Self::new();
        // for v in value {
        //     result
        //         .content
        //         .try_insert(v.id, v.count)
        //         .map_err(|_| DuplicateItemError)?;
        // }
        // Ok(result)
    }
}

impl ItemStorage {
    pub fn new(capacity: ItemCount) -> Self {
        Self {
            content: BTreeMap::new(),
            capacity,
            total_count: 0,
        }
    }

    pub fn capacity(&self) -> ItemCount {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    pub fn free_space(&self) -> ItemCount {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        self.capacity - self.total_count
    }

    /// returns the rest that did not fit inside storage space
    pub fn add_item(&mut self, _item: Item) -> Item {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    /// returns true if an item was added, false if not due to full storage
    pub fn try_add_item(&mut self, _item: Item) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    /// remove as many items as possible
    pub fn remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> Item {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    /// returns true if an item was removed, false if not due to not enough item count in storage
    pub fn try_remove_item(&mut self, _item: Item) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    pub fn count(&self, _item_id: ItemId) -> ItemCount {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    pub fn contains(&self, item: &Item) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        match self.content.get(item.id()) {
            None => false,
            Some(count) => *count >= item.count(),
        }
    }

    pub fn contains_for_input(&self, input: InputRecipe) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        input.into_iter().all(|item| self.contains(&item))
    }

    pub fn try_consume(&mut self, input: InputRecipe) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        let ok = self.contains_for_input(input.clone());
        if !ok {
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

    fn eval_total_count(content: &BTreeMap<ItemId, ItemCount>) -> ItemCount {
        content.iter().map(|(_, count)| count).sum()
    }
}
