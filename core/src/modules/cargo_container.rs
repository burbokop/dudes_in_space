use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId,
    ProcessTokenContext, TradingConsole,
};
use dudes_in_space_api::person::{Logger, ObjectiveDeciderVault, Person, PersonId};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputRecipe, ModuleFactory, Recipe};
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{from_intermediate, to_intermediate, Intermediate};
use std::error::Error;
use std::fmt::{Debug};

static TYPE_ID: &str = "CargoContainer";
static FACTORY_TYPE_ID: &str = "CargoContainerFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];

#[derive(Debug, Serialize, Deserialize)]
struct CargoContainer {
    id: ModuleId,
    storage: ItemStorage
}

impl DynSerialize for CargoContainer {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()  
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl Module for CargoContainer {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
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

    fn free_person_slots_count(&self) -> usize {
        todo!()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        todo!()
    }

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

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn docking_connectors(&self) -> &[DockingConnector] {
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

pub(crate) struct CargoContainerDynSeed;

impl DynDeserializeSeed<dyn Module> for CargoContainerDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: CargoContainer = from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CargoContainerFactory {}

impl ModuleFactory for CargoContainerFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, recipe: &InputRecipe) -> Box<dyn Module> {
        Box::new(CargoContainer{ id: ModuleId::new_v4(), storage: ItemStorage::new() })
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn output_primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }
}

impl DynSerialize for CargoContainerFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()   
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct CargoContainerFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for CargoContainerFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<CargoContainerFactory> =
            from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
