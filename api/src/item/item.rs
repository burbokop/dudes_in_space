use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::{Rc, Weak};
use crate::utils::math::Rational;

pub type ItemId = String;
pub type ItemCount = u32;
pub type ItemWeight = u32;
pub type ItemDensity = Rational<u32>;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Item {
    pub(crate) id: ItemId,
    pub(crate) weight: ItemWeight,
    pub(crate) density: ItemDensity,
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub(crate) item: Weak<Item>,
    pub(crate) count: ItemCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRefStack {
    pub(crate) id: ItemId,
    pub(crate) count: ItemCount,
}

impl ItemRefStack {
    pub fn new(
        id: ItemId,
        count: ItemCount,
    ) -> Self {
        Self { id, count }
    }
}

impl Serialize for ItemStack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Impl<'id> {
            id: &'id ItemId,
            count: ItemCount,
        }

        Impl {
            id: &self.item.upgrade().unwrap().id,
            count: self.count,
        }
        .serialize(serializer)
    }
}

pub(crate) struct ItemStackSeed<'v> {
    vault: &'v ItemVault,
}

impl<'v> ItemStackSeed<'v> {
    fn new(vault: &'v ItemVault) -> Self {
        Self { vault }
    }
}

impl<'de, 'v> DeserializeSeed<'de> for ItemStackSeed<'v> {
    type Value = ItemStack;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Impl {
            id: ItemId,
            count: ItemCount,
        }

        let Impl { id, count } = Impl::deserialize(deserializer)?;
        Ok(Self::Value {
            item: self.vault.get_impl(id).map_err(serde::de::Error::custom)?
            ,
            count,
        })
    }
}

impl ItemStack {
    pub fn new(
        vault: &ItemVault,
        id: ItemId,
        count: ItemCount,
    ) -> Result<Self, ItemNotFoundInVaultError> {
        Ok(Self {
            item: vault.get_impl(id)?,
            count,
        })
    }
    pub fn id(&self) -> ItemId {
        self.item.upgrade().unwrap().id.clone()
    }
    pub fn count(&self) -> ItemCount {
        self.count
    }
}

pub struct ItemVault {
    data: Vec<Rc<Item>>,
}

impl ItemVault {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub(crate) fn get_impl(&self, id: ItemId) -> Result<Weak<Item>, ItemNotFoundInVaultError> {
        self
            .data
            .iter()
            .find(|item| item.id == id)
            .map(Rc::downgrade)
            .ok_or(ItemNotFoundInVaultError)
    }

    fn with(mut self, item: Item) -> Self {
        self.data.push(Rc::new(item));
        self
    }
}

#[derive(Debug)]
pub struct DuplicateItemError;

impl Display for DuplicateItemError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for DuplicateItemError {}

#[derive(Debug)]
pub struct ItemNotFoundInVaultError;

impl Display for ItemNotFoundInVaultError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ItemNotFoundInVaultError {}
