use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemStorage, ItemStorageSeed, ItemVault, ItemVolume};
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, PackageId, TradingConsole,
};
use dudes_in_space_api::person::{
    Logger, ObjectiveDeciderVault, Person, PersonId, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
};
use dudes_in_space_api::utils::physics::M3;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

static TYPE_ID: &str = "CargoContainer";
static FACTORY_TYPE_ID: &str = "CargoContainerFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];
static ITEM_STORAGE_CAPACITY: ItemVolume = M3(1000);

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::cargo_container::CargoContainerSeed::<'context>)]
struct CargoContainer {
    id: ModuleId,
    #[deserialize_seed_xxx(seed = self.seed.storage_seed)]
    storage: ItemStorage,
}

struct CargoContainerSeed<'v> {
    storage_seed: ItemStorageSeed<'v>,
}

impl<'v> CargoContainerSeed<'v> {
    fn new(vault: &'v ItemVault) -> Self {
        Self {
            storage_seed: ItemStorageSeed::new(vault),
        }
    }
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
        environment_context: &mut EnvironmentContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        collector.exit_module();
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
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

    fn storages(&self) -> &[ItemStorage] {
        std::slice::from_ref(&self.storage)
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

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }

    fn docking_connectors(&self) -> &[DockingConnector] {
        &[]
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

pub(crate) struct CargoContainerDynSeed {
    vault: Rc<ItemVault>,
}

impl CargoContainerDynSeed {
    pub(crate) fn new(vault: Rc<ItemVault>) -> Self {
        Self { vault }
    }
}

impl DynDeserializeSeed<dyn Module> for CargoContainerDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: CargoContainer =
            from_intermediate_seed(CargoContainerSeed::new(&self.vault), &intermediate)
                .map_err(|e| e.to_string())?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CargoContainerFactory {}

impl ModuleFactory for CargoContainerFactory {
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(CargoContainer {
            id: ModuleId::new_v4(),
            storage: ItemStorage::new(ITEM_STORAGE_CAPACITY),
        })
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        todo!()
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
