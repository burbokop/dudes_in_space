use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::module::{Module, ModuleCapability, ModuleConsole, ModuleId, ModuleStorage, ModuleTypeId, PackageId, ProcessTokenContext, TradingConsole};
use dudes_in_space_api::person::{ObjectiveDeciderVault, Person, PersonId};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputRecipe, ModuleFactory, Recipe};
use dudes_in_space_api::vessel::{DockingClamp, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;

static TYPE_ID: &str = "Shuttle";
static FACTORY_TYPE_ID: &str = "ShuttleFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, Deserialize)]
struct Shuttle {
    id: ModuleId,
}

impl DynSerialize for Shuttle {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl Module for Shuttle {
    fn storages(&self) -> &[ItemStorage] {
        todo!()
    }

    fn storages_mut(&mut self) -> &mut [ItemStorage] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        todo!()
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn proceed(
        &mut self,
        v: &dyn VesselModuleInterface,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
    ) {
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

    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

pub(crate) struct ShuttleDynSeed;

impl DynDeserializeSeed<dyn Module> for ShuttleDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        _: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Shuttle = from_intermediate(&intermediate).map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ShuttleFactory {}

impl DynSerialize for ShuttleFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct ShuttleFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for ShuttleFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
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

    fn create(&self, _: &InputRecipe) -> Box<dyn Module> {
        Box::new(Shuttle {
            id: ModuleId::new_v4(),
        })
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }
}
