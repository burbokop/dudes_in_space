use std::cmp::Ordering;
use crate::item::{DuplicateItemError, Item, ItemCount, ItemId, ItemRefStack, ItemStack, ItemVault};
use crate::recipe::InputRecipe;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::rc::Weak;
use serde::de::DeserializeSeed;

#[derive(Debug, Clone)]
pub struct ItemStorage {
    content: BTreeMap<ItemId, ItemStack>,
    capacity: ItemCount,
    total_count: ItemCount,
}

#[derive(Clone)]
pub struct ItemStorageSeed<'v> {
    vault: &'v ItemVault,
}

impl<'v> ItemStorageSeed<'v> {
    pub fn new(vault: &'v ItemVault) -> Self {
        Self { vault }
    }
}

impl Serialize for ItemStorage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        todo!()
    }
}

impl<'de, 'v> DeserializeSeed<'de> for ItemStorageSeed<'v> {
    type Value = ItemStorage;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Impl {
            content: BTreeMap<ItemId, ItemCount>,
            capacity: ItemCount,
        }

        let Impl { content, capacity } = Impl::deserialize(deserializer)?;

        let mut c = BTreeMap::new();
        for (k,v) in content {
            c.insert(k.clone(), ItemStack::new(self.vault, k, v).map_err(serde::de::Error::custom)?);
        }
        let total_count = Self::Value::eval_total_count(&c);

        Ok(Self::Value {
            content: c,
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

impl ItemStorage {
    pub fn new(capacity: ItemCount) -> Self {
        Self {
            content: BTreeMap::new(),
            capacity,
            total_count: 0,
        }
    }

    pub fn try_from_vec(value: Vec<ItemStack>, capacity: ItemCount) -> Result<Self, DuplicateItemError> {
        let mut result = Self::new(capacity);
        for v in value {
            result
                .content
                .try_insert(v.item.upgrade().unwrap().id.clone(), v)
                .map_err(|_| DuplicateItemError)?;
        }
        result.total_count = Self::eval_total_count(&result.content);
        Ok(result)
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
    pub fn add(&mut self, _stack: ItemStack) -> ItemStack {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    /// returns true if an item was added, false if not due to full storage
    pub fn try_add_item(&mut self, _item: ItemStack) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    /// remove as many items as possible
    pub fn remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> ItemStack {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    /// returns true if an item was removed, false if not due to not enough item count in storage
    pub fn try_remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    pub fn count(&self, _item_id: ItemId) -> ItemCount {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        todo!()
    }

    pub fn contains(&self, id: ItemId, count: ItemCount) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        match self.content.get(&id) {
            None => false,
            Some(stack) => stack.count >= count,
        }
    }

    pub fn contains_for_input(&self, input: InputRecipe) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        input.into_iter().all(|ItemRefStack { id, count }| self.contains(id, count))
    }

    pub fn try_consume(&mut self, input: InputRecipe) -> bool {
        debug_assert_eq!(self.total_count, Self::eval_total_count(&self.content));
        debug_assert!(self.total_count <= self.capacity);
        let ok = self.contains_for_input(input.clone());
        if !ok {
            return false;
        }

        for ItemRefStack { id, count } in input.into_iter() {
            let stack = self.content.get_mut(&id).unwrap();
            stack.count -= count;
            if stack.count == 0 {
                self.content.remove(&id);
            }
        }

        true
    }

    fn eval_total_count(content: &BTreeMap<ItemId, ItemStack>) -> ItemCount {
        content.iter().map(|(_, stack)| stack.count).sum()
    }
}

#[derive(Debug, Clone)]
struct ItemStorageKey(Weak<Item>);

impl PartialEq for ItemStorageKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr()== other.0.as_ptr()
    }
}

impl Eq for ItemStorageKey {

}

impl PartialOrd for ItemStorageKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.as_ptr().partial_cmp(&other.0.as_ptr())
    }
}

impl Ord for ItemStorageKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_ptr().cmp(&other.0.as_ptr())
    }
}