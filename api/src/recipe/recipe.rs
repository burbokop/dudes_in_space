use crate::item::{DuplicateItemError, Item, ItemCount, ItemId, ItemRefStack};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, btree_map};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemRecipe {
    pub input: InputItemRecipe,
    pub output: OutputItemRecipe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputItemRecipe {
    #[serde(flatten)]
    input: BTreeMap<ItemId, ItemCount>,
}

pub struct InputItemRecipeIntoIter {
    i: btree_map::IntoIter<ItemId, ItemCount>,
}

impl Iterator for InputItemRecipeIntoIter {
    type Item = ItemRefStack;

    fn next(&mut self) -> Option<Self::Item> {
        match self.i.next() {
            None => None,
            Some((id, count)) => Some(ItemRefStack { id, count }),
        }
    }
}

impl IntoIterator for InputItemRecipe {
    type Item = ItemRefStack;
    type IntoIter = InputItemRecipeIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        InputItemRecipeIntoIter {
            i: self.input.into_iter(),
        }
    }
}

impl TryFrom<Vec<ItemRefStack>> for InputItemRecipe {
    type Error = DuplicateItemError;

    fn try_from(value: Vec<ItemRefStack>) -> Result<Self, Self::Error> {
        let mut result = Self {
            input: BTreeMap::new(),
        };
        for v in value {
            result
                .input
                .try_insert(v.id, v.count)
                .map_err(|_| DuplicateItemError)?;
        }
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputItemRecipe {
    #[serde(flatten)]
    output: BTreeMap<ItemId, ItemCount>,
}

impl OutputItemRecipe {
    pub fn len(&self) -> usize {
        self.output.len()
    }
    
    pub fn first(&self) -> Option<(&ItemId, &ItemCount)> {
        self.output.first_key_value()
    }
}

impl IntoIterator for OutputItemRecipe {
    type Item = (ItemId, ItemCount);
    type IntoIter = btree_map::IntoIter<ItemId, ItemCount>;

    fn into_iter(self) -> Self::IntoIter {
        self.output.into_iter()
    }
}

impl TryFrom<Vec<Item>> for OutputItemRecipe {
    type Error = DuplicateItemError;

    fn try_from(_value: Vec<Item>) -> Result<Self, Self::Error> {
        todo!()
    }
}
