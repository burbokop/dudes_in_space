use crate::item::{DuplicateItemError, Item, ItemCount, ItemId, ItemRefStack, ItemStorage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, btree_map};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemRecipe {
    pub input: InputItemRecipe,
    pub output: OutputItemRecipe,
}

#[derive(Debug)]
pub enum CraftingError {
    DoesNotContainItemForInput,
    HasNoSpaceForOutput,
}

impl Display for CraftingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftingError {}

impl ItemRecipe {
    pub fn craft(
        &self,
        input_storage: &mut ItemStorage,
        output_storage: &mut ItemStorage,
    ) -> Result<(), CraftingError> {
        input_storage
            .contains_for_input(self.input.clone())
            .ok_or(CraftingError::DoesNotContainItemForInput)?;
        output_storage
            .has_space_for_output(self.output.clone())
            .ok_or(CraftingError::HasNoSpaceForOutput)?;

        let ok = input_storage.try_consume(self.input.clone());
        assert!(ok);
        let ok = output_storage.try_insert_output(self.output.clone());
        assert!(ok);
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputItemRecipe {
    #[serde(flatten)]
    input: BTreeMap<ItemId, ItemCount>,
}

impl<const N: usize> From<[(ItemId, ItemCount); N]> for InputItemRecipe {
    fn from(arr: [(ItemId, ItemCount); N]) -> Self {
        Self {
            input: BTreeMap::from(arr),
        }
    }
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

impl<const N: usize> From<[(ItemId, ItemCount); N]> for OutputItemRecipe {
    fn from(arr: [(ItemId, ItemCount); N]) -> Self {
        Self {
            output: BTreeMap::from(arr),
        }
    }
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
