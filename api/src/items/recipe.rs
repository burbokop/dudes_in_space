use crate::items::Item;
use serde::{Deserialize, Serialize};

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
        Self { input: value }
    }
}

pub(crate) struct OutputRecipe {
    output: Vec<Item>,
}

impl From<Vec<Item>> for OutputRecipe {
    fn from(value: Vec<Item>) -> Self {
        Self { output: value }
    }
}
