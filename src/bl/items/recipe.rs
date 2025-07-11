use serde::{Deserialize, Serialize};
use crate::bl::Item;

pub(crate) struct Recipe {
    input: Vec<Item>,
    output: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InputRecipe {
    input: Vec<Item>,
}

pub(crate) struct OutputRecipe {
    output: Vec<Item>,
}