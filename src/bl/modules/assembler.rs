use crate::bl::modules::{AssemblyRecipe, AssemblyRecipeSeed, Module, ModuleCapability, ModuleFactory, ModuleVisitor, PersonnelArea, VesselPersonInterface, WorkerControlPanel};
use crate::bl::utils::dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, from_intermediate_seed, VecSeed};
use crate::bl::{Person, Recipe};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use serde_intermediate::Intermediate;
use std::error::Error;
use std::rc::Rc;
use dudes_in_space_macro::DeserializeSeedXXX;

static TYPE_ID: &str = "Assembler";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::bl::modules::assembler::AssemblerSeed::<'v>)]
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

// #[derive(Clone)]
// struct AssemblyRecipeSeqSeed<'v> {
//     seed_vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>,
// }
// 
// impl<'de, 'v> DeserializeSeed<'de> for AssemblyRecipeSeqSeed<'v> {
//     type Value = Vec<AssemblyRecipe>;
// 
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>
//     {
//         
//         todo!()
//     }
// }
//
// impl<'de, 'v> DeserializeSeed<'de> for AssemblerSeed<'v> {
//     type Value = Assembler;
//
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(DeserializeField)]
//         #[deserialize_field(output = crate::bl::modules::assembler::Assembler, context = crate::bl::modules::assembler::ModuleFactoryDynDeserializeSeedVault)]
//         enum Field {
//             Operator,
//             #[deserialize_field(seed = crate::bl::modules::assembler::AssemblyRecipeSeqSeed)]
//             Recipes,
//         }
//
//         todo!()
//     }
// }

impl WorkerControlPanel for Assembler {}

impl Assembler {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            operator: None,
            recipes: vec![],
        })
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
    fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        if let Some(operator) = &mut self.operator {
            operator.proceed(v)
        }
    }

    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
        v.visit_assembler(self)
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        &[ModuleCapability::Crafting]
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> Vec<AssemblyRecipe> {
        todo!()
    }
}

pub(crate) struct AssemblerDynSeed {
    seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
}

impl AssemblerDynSeed {
    pub(crate) fn new(seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>) -> Self {
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
