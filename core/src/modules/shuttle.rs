use crate::modules::PersonnelArea;
use dudes_in_space_api::modules::{
    AssemblyRecipe, Module, ModuleCapability, ModuleFactory, ModuleId, ModuleTypeId, PackageId,
    VesselPersonInterface,
};
use dudes_in_space_api::{InputRecipe, Person, PersonId, Recipe};
use dyn_serde::{DynDeserializeSeed, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;

static TYPE_ID: &str = "Shuttle";
static FACTORY_TYPE_ID: &str = "ShuttleFactory";

#[derive(Debug)]
struct Shuttle {}

impl DynSerialize for Shuttle {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl Module for Shuttle {
    fn id(&self) -> ModuleId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn recipes(&self) -> Vec<Recipe> {
        todo!()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        todo!()
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        todo!()
    }

    fn insert_person(&mut self, person: Person) -> bool {
        todo!()
    }

    fn can_insert_person(&self) -> bool {
        todo!()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        todo!()
    }
}

pub struct ShuttleDynSeed;

impl DynDeserializeSeed<dyn Module> for ShuttleDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate) -> Result<Box<dyn Module>, Box<dyn Error>> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ShuttleFactory {}

impl DynSerialize for ShuttleFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub struct ShuttleFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for ShuttleFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<ShuttleFactory> =
            serde_intermediate::from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}

impl ModuleFactory for ShuttleFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, recipe: &InputRecipe) -> Box<dyn Module> {
        todo!()
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        &[
            ModuleCapability::Cockpit,
            ModuleCapability::Engine,
            ModuleCapability::Reactor,
            ModuleCapability::FuelTank,
        ]
    }
}
