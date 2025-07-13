use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use serde_intermediate::Intermediate;
use std::error::Error;
use std::rc::Rc;
use dudes_in_space_api::modules::{AssemblyRecipe, AssemblyRecipeSeed, Module, ModuleCapability, ModuleFactory, PackageId, VesselPersonInterface, WorkerControlPanel};
use dudes_in_space_api::{Person, Recipe};
use dyn_serde::{from_intermediate_seed, DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, VecSeed};
use dyn_serde_macro::DeserializeSeedXXX;
use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut};

static TYPE_ID: &str = "Assembler";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::assembler::AssemblerSeed::<'v>)]
pub struct Assembler {
    operator: Option<Person>,
    #[deserialize_seed_xxx(seed = self.seed.assembly_recipe_seq_seed)]
    recipes: Vec<AssemblyRecipe>,
}

#[derive(Clone)]
struct AssemblerSeed<'v> {
    assembly_recipe_seq_seed: VecSeed<AssemblyRecipeSeed<'v>>,
}

impl<'v> AssemblerSeed<'v> {
    pub(crate) fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self { assembly_recipe_seq_seed: VecSeed::new(AssemblyRecipeSeed::new(vault)) }
    }
}

impl WorkerControlPanel for Assembler {}

impl Assembler {
    pub(crate) fn new(recipes: Vec<AssemblyRecipe>) -> Box<Self> {
        Box::new(Self {
            operator: None,
            recipes,
        })
    }

    pub fn add_recipe(&mut self, recipe: AssemblyRecipe) {
        self.recipes.push(recipe);
    }
}

impl DynSerialize for Assembler {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        serde_intermediate::to_intermediate(self).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl Module for Assembler {
    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        if let Some(operator) = &mut self.operator {
            operator.proceed(v)
        }
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        &[ModuleCapability::Crafting]
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &self.recipes
    }
}

impl CoreModule for Assembler {
    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
        v.visit_assembler(self)
    }

    fn accept_visitor_mut(&mut self, v: &dyn ModuleVisitorMut<Result = ()>) -> Option<()> {
        v.visit_assembler(self)
    }

}

pub struct AssemblerDynSeed {
    seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
}

impl AssemblerDynSeed {
    pub fn new(seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>) -> Self {
        Self { seed_vault }
    }
}

impl DynDeserializeSeed<dyn Module> for AssemblerDynSeed {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let assembler: Assembler = from_intermediate_seed(
            AssemblerSeed::new(&self.seed_vault) ,
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(assembler))
    }
}
