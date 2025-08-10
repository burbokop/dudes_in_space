use crate::item::{Item, ItemCount, ItemId, ItemRefStack, ItemStack, ItemVault, ItemVolume};
use crate::recipe::InputRecipe;
use crate::utils::physics::M3;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Weak;

#[derive(Debug, Clone)]
pub struct ItemStorage {
    content: BTreeMap<ItemId, ItemStack>,
    volume: ItemVolume,
    total_occupied_volume: ItemVolume,
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
        S: Serializer,
    {
        #[derive(Serialize)]
        pub struct Impl<'a> {
            content: BTreeMap<&'a ItemId, ItemCount>,
            volume: ItemVolume,
        }

        let mut content: BTreeMap<&ItemId, ItemCount> = BTreeMap::new();
        for (k, v) in &self.content {
            content.insert(k, v.count);
        }

        Impl {
            content,
            volume: self.volume,
        }
        .serialize(serializer)
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
            volume: ItemVolume,
        }

        let Impl { content, volume } = Impl::deserialize(deserializer)?;

        let mut c = BTreeMap::new();
        for (k, v) in content {
            c.insert(
                k.clone(),
                ItemStack::new(self.vault, k, v).map_err(serde::de::Error::custom)?,
            );
        }
        let total_occupied_volume = Self::Value::eval_total_occupied_volume(&c);

        Ok(Self::Value {
            content: c,
            volume,
            total_occupied_volume,
        })
    }
}

impl FromIterator<Item> for ItemStorage {
    fn from_iter<T: IntoIterator<Item = Item>>(_iter: T) -> Self {
        todo!()
    }
}

impl ItemStorage {
    pub fn new(volume: ItemVolume) -> Self {
        Self {
            content: BTreeMap::new(),
            volume,
            total_occupied_volume: M3(0),
        }
    }

    pub fn from_vec(
        value: Vec<ItemStack>,
        volume: ItemVolume,
    ) -> Result<Self, ItemStorageFromVecError> {
        let mut result = Self::new(volume);
        for v in value {
            result
                .content
                .try_insert(v.item.upgrade().unwrap().id.clone(), v)
                .map_err(|_| ItemStorageFromVecError::DuplicateItem)?;
        }
        result.total_occupied_volume = Self::eval_total_occupied_volume(&result.content);

        if result.total_occupied_volume > volume {
            Err(ItemStorageFromVecError::VolumeExceeded)
        } else {
            Ok(result)
        }
    }

    pub fn capacity(&self) -> ItemCount {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        todo!()
    }

    pub fn free_space(&self) -> ItemVolume {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        self.volume - self.total_occupied_volume
    }

    /// returns the rest that did not fit inside storage space
    pub fn add(&mut self, _stack: ItemStack) -> ItemStack {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        todo!()
    }

    /// returns true if an item was added, false if not due to full storage
    pub fn try_add_item(&mut self, _item: ItemStack) -> bool {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        todo!()
    }

    /// remove as many items as possible
    pub fn remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> ItemStack {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        todo!()
    }

    /// returns true if an item was removed, false if not due to not enough item count in storage
    pub fn try_remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> bool {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        todo!()
    }

    pub fn count(&self, _item_id: ItemId) -> ItemCount {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        todo!()
    }

    pub fn contains(&self, id: ItemId, count: ItemCount) -> bool {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        match self.content.get(&id) {
            None => false,
            Some(stack) => stack.count >= count,
        }
    }

    pub fn contains_for_input(&self, input: InputRecipe) -> bool {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
        input
            .into_iter()
            .all(|ItemRefStack { id, count }| self.contains(id, count))
    }

    pub fn try_consume(&mut self, input: InputRecipe) -> bool {
        debug_assert_eq!(
            self.total_occupied_volume,
            Self::eval_total_occupied_volume(&self.content)
        );
        debug_assert!(self.total_occupied_volume <= self.volume);
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

    fn eval_total_occupied_volume(content: &BTreeMap<ItemId, ItemStack>) -> ItemVolume {
        content.iter().map(|(_, stack)| stack.volume()).sum()
    }
}

#[derive(Debug, Clone)]
struct ItemStorageKey(Weak<Item>);

impl PartialEq for ItemStorageKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl Eq for ItemStorageKey {}

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

#[derive(Debug)]
pub enum ItemStorageFromVecError {
    DuplicateItem,
    VolumeExceeded,
}

impl Display for ItemStorageFromVecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ItemStorageFromVecError {}
