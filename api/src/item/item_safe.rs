



use crate::item::{ItemCount, ItemId, 
                  ItemStack, ItemStorage, ItemStorageSeed, ItemVault, ItemVolume};
use crate::recipe::{InputItemRecipe, OutputItemRecipe};
use serde::{ 
    Serialize, 
};

use std::collections::BTreeMap;



use dyn_serde::{MapSeed, 
};
use dyn_serde_macro::DeserializeSeedXXX;
use crate::person::PersonId;

#[derive(Debug, Clone, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::item::item_safe::ItemSafeSeed::<'v>)]
pub struct ItemSafe {
    #[deserialize_seed_xxx(seed = self.seed.item_storage_seed)]
    content: BTreeMap<PersonId, ItemStorage>,
    persons_capacity: usize,
}

#[derive(Clone)]
pub struct ItemSafeSeed<'v> {
    item_storage_seed: MapSeed<PersonId, ItemStorageSeed<'v>>,
}

impl<'v> ItemSafeSeed<'v> {
    pub fn new(vault: &'v ItemVault) -> Self {
        Self { item_storage_seed: MapSeed::new(ItemStorageSeed::new(vault)) }
    }
}

impl ItemSafe {
    pub fn new(persons_capacity: usize) -> Self {
        Self {
            content: BTreeMap::new(),
            persons_capacity,
        }
    }
    
    pub fn capacity(&self, person_id: PersonId) -> ItemCount {
        todo!()
    }

    pub fn free_space(&self, person_id: PersonId) -> ItemVolume {
        todo!()
    }

    /// returns the rest that did not fit inside storage space
    pub fn add(&mut self,person_id: PersonId, _stack: ItemStack) -> ItemStack {
        todo!()
    }

    /// returns true if an item was added, false if not due to full storage
    pub fn try_add_item(&mut self, person_id: PersonId, _item: ItemStack) -> bool {
        todo!()
    }

    /// remove as many items as possible
    pub fn remove_item(&mut self,person_id: PersonId, _item_id: ItemId, _count: ItemCount) -> ItemStack {
        todo!()
    }

    /// returns true if an item was removed, false if not due to not enough item count in storage
    pub fn try_remove_item(&mut self,person_id: PersonId, _item_id: ItemId, _count: ItemCount) -> bool {
        todo!()
    }

    pub fn count(&self,person_id: PersonId, _item_id: ItemId) -> ItemCount {
        todo!()
    }

    pub fn contains(&self,person_id: PersonId, id: ItemId, count: ItemCount) -> bool {
        todo!()
    }

    pub fn contains_for_input(&self,person_id: PersonId, input: InputItemRecipe) -> bool {
        todo!()
    }

    pub fn try_consume(&mut self,person_id: PersonId, input: InputItemRecipe) -> bool {
        todo!()
    }

    pub fn has_space_for_output(&self,person_id: PersonId, output: OutputItemRecipe) -> bool {
        todo!()
    }

    pub fn try_insert_output(&mut self,person_id: PersonId, output: OutputItemRecipe) -> bool {
        todo!()
    }
}
