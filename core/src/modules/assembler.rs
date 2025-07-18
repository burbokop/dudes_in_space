use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut, ShuttleFactory};
use dudes_in_space_api::modules::{AssemblyRecipe, AssemblyRecipeSeed, Module, ModuleCapability, ModuleFactory, ModuleId, PackageId, VesselPersonInterface, WorkerControlPanel};
use dudes_in_space_api::{Item, Person, PersonId, Recipe};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, VecSeed, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::{Serialize};
use serde_intermediate::{to_intermediate, Intermediate};
use std::error::Error;
use std::rc::Rc;

static TYPE_ID: &str = "Assembler";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::assembler::AssemblerSeed::<'v>)]
pub struct Assembler {
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    operator: Option<Person>,
    #[deserialize_seed_xxx(seed = self.seed.assembly_recipe_seq_seed)]
    recipes: Vec<AssemblyRecipe>,
    id: ModuleId,
}

#[derive(Clone)]
pub struct AssemblerSeed<'v> {
    assembly_recipe_seq_seed: VecSeed<AssemblyRecipeSeed<'v>>,
}

impl<'v> AssemblerSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self {
            assembly_recipe_seq_seed: VecSeed::new(AssemblyRecipeSeed::new(vault)),
        }
    }
}

impl WorkerControlPanel for Assembler {}

impl Assembler {
    pub fn new(recipes: Vec<AssemblyRecipe>) -> Box<Self> {
        Box::new(Self {
            operator: None,
            recipes,
            id: ModuleId::new_v4()
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
        to_intermediate(self).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl Module for Assembler {
    fn id(&self) -> ModuleId {
        self.id
    }

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

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        todo!()
    }

    fn insert_person(&mut self, person: Person) -> bool {
        if self.operator.is_none() {
            self.operator = Some(person);
            true
        } else {
            false
        }
    }

    fn can_insert_person(&self) -> bool {
        self.operator.is_none()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        todo!()
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
        let obj: Assembler = from_intermediate_seed(AssemblerSeed::new(&self.seed_vault), &intermediate)
                .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}
