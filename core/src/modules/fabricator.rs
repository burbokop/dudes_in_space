use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemStorage, ItemStorageSeed, ItemVault, ItemVolume};
use dudes_in_space_api::module::{
    CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, ModuleTypeId, PackageId, ProcessTokenContext, ProcessTokenMut,
    ProcessTokenMutSeed, TradingAdminConsole, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory};
use dudes_in_space_api::utils::physics::M3;
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::convert::Into;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

static TYPE_ID: &str = "Fabricator";
static FACTORY_TYPE_ID: &str = "FabricatorFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::ItemCrafting,
    ModuleCapability::ItemStorage,
    ModuleCapability::PersonnelRoom,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemCrafting];
static ITEM_STORAGE_CAPACITY: ItemVolume = M3(100);

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

impl<'v, 'sv, 'context> FabricatorSeed<'v, 'sv, 'context> {
    fn new(
        objective_seed_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        item_vault: &'sv ItemVault,
        context: &'context ProcessTokenContext,
    ) -> Self {
        Self {
            person_seed: TaggedOptionSeed::new(PersonSeed::new(objective_seed_vault)),
            item_storage_seed: ItemStorageSeed::new(item_vault),
            state_seed: FabricatorStateSeed::new(context),
        }
    }
}

impl DynSerialize for Fabricator {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

enum FabricatorRequest {
    Interact,
}

struct Console<'a> {
    id: ModuleId,
    recipes: &'a [ItemRecipe],
    requests: Vec<FabricatorRequest>,
    state: &'a mut FabricatorState,
    input_storage: &'a mut ItemStorage,
    output_storage: &'a mut ItemStorage,
}

impl ModuleConsole for Console<'_> {
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

    fn interact(&mut self) -> bool {
        todo!()
    }

    fn in_progress(&self) -> bool {
        todo!()
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        todo!()
    }

    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole> {
        todo!()
    }

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole> {
        todo!()
    }

    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole> {
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }

    fn trading_admin_console(&self) -> Option<&dyn TradingAdminConsole> {
        todo!()
    }

    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn TradingAdminConsole> {
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
}

impl Module for Fabricator {
    fn id(&self) -> ModuleId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
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
        let mut console = Console {
            id: self.id,
            recipes: &self.recipes,
            requests: vec![],
            state: &mut self.state,
            input_storage: &mut self.input_storage,
            output_storage: &mut self.output_storage,
        };

        if let Some(operator) = &mut self.operator {
            operator.proceed(
                &mut rng(),
                &mut console,
                this_vessel.console(),
                environment_context,
                decider_vault,
                logger,
            )
        }

        for request in std::mem::take(&mut console.requests) {
            match request {
                FabricatorRequest::Interact => match &self.state {
                    FabricatorState::Idle => todo!(),
                    FabricatorState::Fabricating {
                        recipe_index,
                        process_token,
                    } => {
                        self.recipes[*recipe_index]
                            .craft(&mut self.input_storage, &mut self.output_storage)
                            .unwrap();
                        self.state = FabricatorState::Idle;
                    }
                },
            }
        }
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

pub(crate) struct FabricatorDynSeed {
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    item_vault: Rc<ItemVault>,
    context: Rc<ProcessTokenContext>,
}

impl FabricatorDynSeed {
    pub(crate) fn new(
        objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        item_vault: Rc<ItemVault>,
        context: Rc<ProcessTokenContext>,
    ) -> Self {
        Self {
            objective_seed_vault,
            item_vault,
            context,
        }
    }
}

impl DynDeserializeSeed<dyn Module> for FabricatorDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Fabricator = from_intermediate_seed(
            FabricatorSeed::new(&self.objective_seed_vault, &self.item_vault, &self.context),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FabricatorFactory {}
struct FabricatorFactorySeed {}

impl Debug for FabricatorFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl DynSerialize for FabricatorFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl ModuleFactory for FabricatorFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(Fabricator {
            id: ModuleId::new_v4(),
            recipes: vec![],
            state: FabricatorState::Idle,
            input_storage: ItemStorage::new(ITEM_STORAGE_CAPACITY),
            output_storage: ItemStorage::new(ITEM_STORAGE_CAPACITY),
            operator: None,
        })
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn output_primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
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
        let r: Box<FabricatorFactory> =
            serde_intermediate::from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
