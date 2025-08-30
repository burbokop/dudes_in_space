use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemSafe, ItemStorage, ItemVolume};
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId, TradingConsole,
};
use dudes_in_space_api::person::{
    Logger, ObjectiveDeciderVault, Person, PersonId, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::utils::physics::M3;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;
use std::sync::LazyLock;

static TYPE_ID: &str = "PlantFacility";
static FACTORY_TYPE_ID: &str = "PlantFacilityFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::ItemProduction,
    ModuleCapability::ItemStorage,
    ModuleCapability::PersonnelRoom,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemProduction];
static ITEM_STORAGE_CAPACITY: ItemVolume = M3(100);
pub(crate) static RECIPES: LazyLock<[OutputItemRecipe; 1]> =
    LazyLock::new(|| [[("biomass".into(), 1)].into()]);

#[derive(Debug)]
struct PlantFacility {}

struct PlantFacilitySeed {}

pub(crate) struct PlantFacilityDynSeed {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlantFacilityFactory {}

pub(crate) struct PlantFacilityFactoryDynSeed;

impl Module for PlantFacility {
    fn id(&self) -> ModuleId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        environment_context: &mut EnvironmentContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        todo!()
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        todo!()
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        todo!()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        todo!()
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
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

    fn persons(&self) -> &[Person] {
        todo!()
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        todo!()
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn safes(&self) -> &[ItemSafe] {
        todo!()
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
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

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
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

impl ModuleFactory for PlantFacilityFactory {
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        todo!()
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for PlantFacilityFactory {
    fn type_id(&self) -> ModuleTypeId {
        TYPE_ID.into()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        RECIPES.as_ref()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }
}

impl DynSerialize for PlantFacility {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl DynSerialize for PlantFacilityFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl DynDeserializeSeed<dyn Module> for PlantFacilityDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        todo!()
    }
}

impl DynDeserializeSeed<dyn ModuleFactory> for PlantFacilityFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<PlantFacilityFactory> =
            from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
