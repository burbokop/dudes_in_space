use crate::items::InputRecipe;
use crate::modules::{Module, ModuleCapability, ModuleTypeId};
use dyn_serde::{DynDeserializeSeedVault, DynSerialize};
use dyn_serde_macro::{DeserializeSeedXXX, dyn_serde_trait};
use serde::Serialize;
use std::fmt::Debug;
use std::rc::Rc;

pub trait ModuleFactory: Debug + DynSerialize {
    fn output_type_id(&self) -> ModuleTypeId;
    fn create(&self, recipe: &InputRecipe) -> Box<dyn Module>;
    fn output_capabilities(&self) -> &[ModuleCapability];
}

dyn_serde_trait!(ModuleFactory);

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::assembly_recipe::AssemblyRecipeSeed::<'v>)]
pub struct AssemblyRecipe {
    input: InputRecipe,
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
    pub fn new(input: InputRecipe, output: Rc<dyn ModuleFactory>) -> Self {
        Self { input, output }
    }

    pub(crate) fn create(&self) -> Box<dyn Module> {
        self.output.create(&self.input)
    }
    pub(crate) fn output_capabilities(&self) -> &[ModuleCapability] {
        self.output.output_capabilities()
    }
}
