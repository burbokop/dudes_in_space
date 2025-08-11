use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemStorage, ItemStorageSeed, ItemVolume};
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId,
    ProcessTokenContext, ProcessTokenMut, ProcessTokenMutSeed, TradingConsole,
};
use dudes_in_space_api::person::{
    Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory};
use dudes_in_space_api::utils::physics::M3;
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use serde_intermediate::Intermediate;
use std::error::Error;
use std::fmt::{Debug, Formatter};

static TYPE_ID: &str = "Fabricator";
static FACTORY_TYPE_ID: &str = "FabricatorFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::ItemCrafting,
    ModuleCapability::ItemStorage,
    ModuleCapability::PersonnelRoom,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemCrafting];
static ITEM_STORAGE_CAPACITY: ItemVolume = M3(1000);

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::fabricator::FabricatorStateSeed::<'context>)]
#[serde(tag = "tp")]
enum FabricatorState {
    Idle,
    #[deserialize_seed_xxx(seeds = [(process_token, self.seed.seed.process_token_seed)])]
    Fabricating {
        recipe_index: usize,
        process_token: ProcessTokenMut,
    },
}

#[derive(Clone)]
struct FabricatorStateSeed<'context> {
    process_token_seed: ProcessTokenMutSeed<'context>,
}

impl<'context> FabricatorStateSeed<'context> {
    fn new(context: &'context ProcessTokenContext) -> Self {
        Self {
            process_token_seed: ProcessTokenMutSeed::new(context),
        }
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::fabricator::FabricatorSeed::<'v, 'sv, 'context>)]
struct Fabricator {
    id: ModuleId,
    recipes: Vec<ItemRecipe>,
    #[deserialize_seed_xxx(seed = self.seed.state_seed)]
    state: FabricatorState,
    #[deserialize_seed_xxx(seed = self.seed.item_storage_seed)]
    input_storage: ItemStorage,
    #[deserialize_seed_xxx(seed = self.seed.item_storage_seed)]
    output_storage: ItemStorage,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}
struct FabricatorSeed<'v, 'sv, 'context> {
    person_seed: TaggedOptionSeed<PersonSeed<'v>>,
    item_storage_seed: ItemStorageSeed<'sv>,
    state_seed: FabricatorStateSeed<'context>,
}

impl DynSerialize for Fabricator {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl Module for Fabricator {
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

pub(crate) struct FabricatorDynSeed;

impl DynDeserializeSeed<dyn Module> for FabricatorDynSeed {
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

pub(crate) struct FabricatorFactory {}
struct FabricatorFactorySeed {}

impl Debug for FabricatorFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl DynSerialize for FabricatorFactory {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl ModuleFactory for FabricatorFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        todo!()
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn output_primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }
}

pub(crate) struct FabricatorFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for FabricatorFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        todo!()
    }
}
