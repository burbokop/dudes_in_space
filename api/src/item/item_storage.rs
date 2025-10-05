use crate::item::{Item, ItemCount, ItemId, ItemRefStack, ItemStack, ItemVault, ItemVolume};
use crate::recipe::{InputItemRecipe, OutputItemRecipe};
use crate::utils::physics::M3;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::rc::{Rc, Weak};

macro_rules! validate {
    ($this:ident) => {
        debug_assert_eq!(
            $this.total_occupied_volume,
            Self::eval_total_occupied_volume(&$this.content.0)
        );
        debug_assert!($this.total_occupied_volume <= $this.volume);
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorageRole {
    Input,
    Output,
    NoRole,
}

#[derive(Debug, Clone)]
pub struct ItemStorageContent(BTreeMap<ItemId, ItemStack>);

impl ItemStorageContent {
    pub fn stacks(&self) -> impl Iterator<Item = &ItemStack> {
        self.0.values()
    }

    pub fn count(&self, id: &ItemId) -> ItemCount {
        self.0.get(id).map(|v| v.count).unwrap_or(0)
    }

    pub fn counts(&self, items: impl Iterator<Item = ItemId>) -> BTreeMap<ItemId, ItemCount> {
        items
            .map(|id| {
                let count = self.count(&id);
                (id, count)
            })
            .collect()
    }

    /// Returns how much is needed to add to this storage to reach the limits
    pub fn lack(&self, mut limits: BTreeMap<ItemId, ItemCount>) -> BTreeMap<ItemId, ItemCount> {
        for (k, v) in &self.0 {
            if !limits.contains_key(k) {
                limits.try_insert(k.clone(), 0).unwrap();
            }
        }

        limits
            .iter()
            .map(|(k, v)| (k.clone(), v.saturating_sub(self.count(k))))
            .collect()
    }
}

impl Sum for ItemStorageContent {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Self(BTreeMap::new());
        for v in iter {
            for (k, stack) in v.0 {
                if let Some(existing) = result.0.get_mut(&k) {
                    existing.count += stack.count;
                } else {
                    result.0.insert(k, stack.clone());
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct ItemStorage {
    content: ItemStorageContent,
    volume: ItemVolume,
    total_occupied_volume: ItemVolume,
    item_vault: Rc<ItemVault>,
}

#[derive(Clone)]
pub struct ItemStorageSeed {
    vault: Rc<ItemVault>,
}

impl ItemStorageSeed {
    pub fn new(vault: Rc<ItemVault>) -> Self {
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
        for (k, v) in &self.content.0 {
            content.insert(k, v.count);
        }

        Impl {
            content,
            volume: self.volume,
        }
        .serialize(serializer)
    }
}

impl<'de> DeserializeSeed<'de> for ItemStorageSeed {
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
                ItemStack::new(&self.vault, k, v).map_err(serde::de::Error::custom)?,
            );
        }
        let total_occupied_volume = Self::Value::eval_total_occupied_volume(&c);

        Ok(Self::Value {
            content: ItemStorageContent(c),
            volume,
            total_occupied_volume,
            item_vault: self.vault.clone(),
        })
    }
}

impl FromIterator<Item> for ItemStorage {
    fn from_iter<T: IntoIterator<Item = Item>>(_iter: T) -> Self {
        todo!()
    }
}

impl ItemStorage {
    pub fn new(item_vault: Rc<ItemVault>, volume: ItemVolume) -> Self {
        Self {
            content: ItemStorageContent(BTreeMap::new()),
            volume,
            total_occupied_volume: M3(0),
            item_vault,
        }
    }

    pub fn content(&self) -> &ItemStorageContent {
        &self.content
    }

    pub fn from_vec(
        item_vault: Rc<ItemVault>,
        value: Vec<ItemStack>,
        volume: ItemVolume,
    ) -> Result<Self, ItemStorageFromVecError> {
        let mut result = Self::new(item_vault, volume);
        for v in value {
            result
                .content
                .0
                .try_insert(v.item.upgrade().unwrap().id.clone(), v)
                .map_err(|_| ItemStorageFromVecError::DuplicateItem)?;
        }
        result.total_occupied_volume = Self::eval_total_occupied_volume(&result.content.0);

        if result.total_occupied_volume > volume {
            Err(ItemStorageFromVecError::VolumeExceeded)
        } else {
            Ok(result)
        }
    }

    pub fn capacity(&self) -> ItemVolume {
        validate!(self);
        self.volume
    }

    pub fn free_space(&self) -> ItemVolume {
        validate!(self);
        self.volume - self.total_occupied_volume
    }

    /// returns the rest that did not fit inside storage space
    pub fn add(&mut self, _stack: ItemStack) -> ItemStack {
        validate!(self);
        todo!()
    }

    /// returns true if an item was added, false if not due to full storage
    pub fn try_add_item(&mut self, _item: ItemStack) -> bool {
        validate!(self);
        todo!()
    }

    /// remove as many items as possible
    pub fn remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> ItemStack {
        validate!(self);
        todo!()
    }

    /// returns true if an item was removed, false if not due to not enough item count in storage
    pub fn try_remove_item(&mut self, _item_id: ItemId, _count: ItemCount) -> bool {
        validate!(self);
        todo!()
    }

    pub fn count(&self, id: ItemId) -> ItemCount {
        validate!(self);
        self.content.0.get(&id).map(|v| v.count).unwrap_or(0)
    }

    pub fn contains(&self, id: ItemId, count: ItemCount) -> bool {
        validate!(self);
        match self.content.0.get(&id) {
            None => false,
            Some(stack) => stack.count >= count,
        }
    }

    pub fn contains_for_input(&self, input: InputItemRecipe) -> bool {
        validate!(self);
        input
            .into_iter()
            .all(|ItemRefStack { id, count }| self.contains(id, count))
    }

    pub fn try_consume(&mut self, input: InputItemRecipe) -> bool {
        validate!(self);
        let ok = self.contains_for_input(input.clone());
        if !ok {
            return false;
        }

        for ItemRefStack { id, count } in input {
            let stack = self.content.0.get_mut(&id).unwrap();
            stack.count -= count;
            if stack.count == 0 {
                self.content.0.remove(&id);
            }
        }

        self.total_occupied_volume = Self::eval_total_occupied_volume(&self.content.0);
        true
    }

    pub fn has_space_for_output(&self, output: OutputItemRecipe) -> bool {
        validate!(self);

        let delta_volume: ItemVolume = output
            .into_iter()
            .map(|(item, count)| {
                let item = self.item_vault.get(item).unwrap();
                let item = item.upgrade().unwrap();
                item.volume * count
            })
            .sum();

        self.free_space() >= delta_volume
    }

    pub fn try_insert_output(&mut self, output: OutputItemRecipe) -> bool {
        validate!(self);
        if !self.has_space_for_output(output.clone()) {
            return false;
        }

        for (id, count) in output {
            // Maybe better skip then. But it is weird to have a recipe that outputs nothing
            assert_ne!(count, 0);

            let stack = self
                .content
                .0
                .entry(id.clone())
                .or_insert_with(|| ItemStack::new(&self.item_vault, id, 0).unwrap());
            stack.count += count;
        }

        self.total_occupied_volume = Self::eval_total_occupied_volume(&self.content.0);
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
