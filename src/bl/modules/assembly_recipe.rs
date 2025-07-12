use crate::bl::modules::{Module, ModuleCapability, ModuleTypeId};
use crate::bl::utils::dyn_serde::{DynDeserializeSeedVault, DynSerialize, VecSeed};
use crate::bl::{InputRecipe, Item};
use dudes_in_space_macro::{dyn_serde_trait, DeserializeSeedXXX};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

pub(crate) trait ModuleFactory: Debug + DynSerialize {
    fn type_id(&self) -> ModuleTypeId;
    fn create(&self, recipe: &InputRecipe) -> Box<dyn Module>;
    fn capabilities(&self) -> &[ModuleCapability];
}

dyn_serde_trait!(ModuleFactory);

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::bl::modules::assembly_recipe::AssemblyRecipeSeed::<'v>)]
pub struct AssemblyRecipe {
    input: InputRecipe,
    #[deserialize_seed_xxx(seed = self.seed.module_factory_seed)]
    output: Rc<dyn ModuleFactory>,
}

#[derive(Clone)]
pub(crate) struct AssemblyRecipeSeed<'v> {
    module_factory_seed: ModuleFactorySeed<'v>,
}

impl<'v> AssemblyRecipeSeed<'v> {
    pub(crate) fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self { module_factory_seed: ModuleFactorySeed::new(vault) }
    }
}

impl AssemblyRecipe {
    pub(crate) fn new(input: InputRecipe, output: Rc<dyn ModuleFactory>) -> Self {
        Self { input, output }
    }

    pub(crate) fn create(&self) -> Box<dyn Module> {
        todo!()
        // self.output.create(&self.input)
    }
    pub(crate) fn output_capabilities(&self) -> &[ModuleCapability] {
        todo!()
        // self.output.capabilities()
    }
}
