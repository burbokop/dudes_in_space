use crate::item::{DuplicateItemError, Item, ItemCount, ItemId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, btree_map};

pub struct Recipe {
    input: Vec<Item>,
    output: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputRecipe {
    #[serde(flatten)]
    input: BTreeMap<ItemId, ItemCount>,
}

pub struct InputRecipeIntoIter {
    i: btree_map::IntoIter<ItemId, ItemCount>,
}

impl Iterator for InputRecipeIntoIter {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.i.next() {
            None => None,
            Some((id, count)) => Some(Item::new(id, count)),
        }
    }
}

impl IntoIterator for InputRecipe {
    type Item = Item;
    type IntoIter = InputRecipeIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        InputRecipeIntoIter {
            i: self.input.into_iter(),
        }
    }
}

impl TryFrom<Vec<Item>> for InputRecipe {
    type Error = DuplicateItemError;

    fn try_from(value: Vec<Item>) -> Result<Self, Self::Error> {
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

pub(crate) struct OutputRecipe {
    output: BTreeMap<ItemId, ItemCount>,
}

impl TryFrom<Vec<Item>> for OutputRecipe {
    type Error = DuplicateItemError;

    fn try_from(value: Vec<Item>) -> Result<Self, Self::Error> {
        todo!()
    }
}
