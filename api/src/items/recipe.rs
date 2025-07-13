use serde::{Deserialize, Serialize};
use crate::items::Item;

pub struct Recipe {
    input: Vec<Item>,
    output: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputRecipe {
    input: Vec<Item>,
}

impl From<Vec<Item>> for InputRecipe {
    fn from(value: Vec<Item>) -> Self {
        todo!()
    }
}

pub(crate) struct OutputRecipe {
    output: Vec<Item>,
}
