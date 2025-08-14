use crate::module::{Module, ModuleCapability, ModuleTypeId};
use crate::recipe::{InputItemRecipe, ItemRecipe, OutputItemRecipe};
use dyn_serde::{DynDeserializeSeedVault, DynSerialize};
use dyn_serde_macro::{DeserializeSeedXXX, dyn_serde_trait};
use serde::Serialize;
use std::fmt::Debug;
use std::rc::Rc;

pub trait ModuleFactoryOutputDescription {
    fn type_id(&self) -> ModuleTypeId;
    fn capabilities(&self) -> &[ModuleCapability];
    fn primary_capabilities(&self) -> &[ModuleCapability];
    fn item_recipes(&self) -> &[ItemRecipe];
    fn input_item_recipes(&self) -> &[InputItemRecipe];
    fn output_item_recipes(&self) -> &[OutputItemRecipe];
    fn assembly_recipes(&self) -> &[AssemblyRecipe];
}

pub trait ModuleFactory: Debug + DynSerialize {
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module>;
    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription;
}

dyn_serde_trait!(ModuleFactory, ModuleFactorySeed);

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::recipe::AssemblyRecipeSeed::<'v>)]
pub struct AssemblyRecipe {
    input: InputItemRecipe,
    #[deserialize_seed_xxx(seed = self.seed.module_factory_seed)]
    output: Rc<dyn ModuleFactory>,
}

#[derive(Clone)]
pub struct AssemblyRecipeSeed<'v> {
    module_factory_seed: ModuleFactorySeed<'v>,
}

impl<'v> AssemblyRecipeSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self {
            module_factory_seed: ModuleFactorySeed::new(vault),
        }
    }
}

impl AssemblyRecipe {
    pub fn new(input: InputItemRecipe, output: Rc<dyn ModuleFactory>) -> Self {
        Self { input, output }
    }

    pub fn input(&self) -> &InputItemRecipe {
        &self.input
    }

    pub fn create(&self) -> Box<dyn Module> {
        self.output.create(&self.input)
    }

    pub fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self.output.output_description()
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX, Clone)]
#[deserialize_seed_xxx(seed = crate::recipe::assembly_recipe::OutputRecipeSeed::<'v>)]
#[serde(tag = "tp")]
pub enum OutputRecipe {
    Item(OutputItemRecipe),
    #[deserialize_seed_xxx(seeds = [(field_0, self.seed.seed.module_factory_seed)])]
    Module(Rc<dyn ModuleFactory>),
}

#[derive(Clone)]
struct OutputRecipeSeed<'v> {
    module_factory_seed: ModuleFactorySeed<'v>,
}

impl<'v> OutputRecipeSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self {
            module_factory_seed: ModuleFactorySeed::new(vault),
        }
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX, Clone)]
#[deserialize_seed_xxx(seed = crate::recipe::assembly_recipe::RecipeSeed::<'v>)]
pub struct Recipe {
    input: InputItemRecipe,
    #[deserialize_seed_xxx(seed = self.seed.output_recipe_seed)]
    output: OutputRecipe,
}

#[derive(Clone)]
struct RecipeSeed<'v> {
    output_recipe_seed: OutputRecipeSeed<'v>,
}

impl<'v> RecipeSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self {
            output_recipe_seed: OutputRecipeSeed::new(vault),
        }
    }
}
