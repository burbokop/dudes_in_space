use crate::CORE_PACKAGE_ID;
use burbomath::physics::M3;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{BankRegistry, WalletRegistry};
use dudes_in_space_api::item::{
    ItemId, ItemSafe, ItemStorage, ItemStorageSeed, ItemVault, ItemVolume, StorageRole,
};
use dudes_in_space_api::module::{
    AdminTradingConsole, CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole,
    ModuleId, ModuleStorage, ModuleTypeId, PackageId, ProcessToken, ProcessTokenContext,
    ProcessTokenMut, ProcessTokenMutSeed, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, AssemblyRecipeSeed, InputItemRecipe, InputItemRecipeHash, ItemRecipe,
    ItemRecipeHash, ModuleFactory, ModuleFactoryOutputDescription, OutputItemRecipe,
    OutputItemRecipeHash,
};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, VecSeed,
    from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::LazyLock;

pub(super) static TYPE_ID: &str = "PlantFacility";
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

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::plant_facility::PlantFacilityStateSeed::<'context>)]
#[serde(tag = "tp")]
enum PlantFacilityState {
    Idle,
    #[deserialize_seed_xxx(seeds = [(process_token, self.seed.seed.process_token_seed)])]
    Producing {
        recipe_index: usize,
        process_token: ProcessTokenMut,
    },
}

#[derive(Clone)]
struct PlantFacilityStateSeed<'context> {
    process_token_seed: ProcessTokenMutSeed<'context>,
}

impl<'context> PlantFacilityStateSeed<'context> {
    fn new(context: &'context ProcessTokenContext) -> Self {
        Self {
            process_token_seed: ProcessTokenMutSeed::new(context),
        }
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::plant_facility::PlantFacilitySeed::<'v,'a,'b, 'context>)]
pub struct PlantFacility {
    id: ModuleId,
    #[deserialize_seed_xxx(seed = self.seed.state_seed)]
    state: PlantFacilityState,
    #[deserialize_seed_xxx(seed = self.seed.item_storage_seed)]
    storage: ItemStorage,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}

impl PlantFacility {
    fn new(storage: ItemStorage) -> Box<Self> {
        Box::new(Self {
            id: ModuleId::new_v4(),
            state: PlantFacilityState::Idle,
            storage,
            operator: None,
        })
    }

    fn with_operator(person: Person, storage: ItemStorage) -> Box<Self> {
        Box::new(Self {
            id: ModuleId::new_v4(),
            state: PlantFacilityState::Idle,
            storage,
            operator: Some(person),
        })
    }
}

#[derive(Clone)]
pub struct PlantFacilitySeed<'v, 'a, 'b, 'context> {
    recipe_seq_seed: VecSeed<AssemblyRecipeSeed<'v>>,
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'a, 'b>>,
    item_storage_seed: ItemStorageSeed,
    state_seed: PlantFacilityStateSeed<'context>,
}

impl<'v, 'a, 'b, 'context> PlantFacilitySeed<'v, 'a, 'b, 'context> {
    pub fn new(
        module_factory_vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'a BankRegistry,
        wallet_registry: &'b WalletRegistry,
        item_vault: Rc<ItemVault>,
        context: &'context ProcessTokenContext,
    ) -> Self {
        Self {
            recipe_seq_seed: VecSeed::new(AssemblyRecipeSeed::new(module_factory_vault)),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(
                objective_vault,
                bank_registry,
                wallet_registry,
            )),
            item_storage_seed: ItemStorageSeed::new(item_vault),
            state_seed: PlantFacilityStateSeed::new(context),
        }
    }
}

pub(crate) struct PlantFacilityDynSeed {
    factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    wallet_registry: Rc<WalletRegistry>,
    item_vault: Rc<ItemVault>,
    context: Rc<ProcessTokenContext>,
}

impl PlantFacilityDynSeed {
    pub(crate) fn new(
        factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
        objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
        wallet_registry: Rc<WalletRegistry>,
        item_vault: Rc<ItemVault>,
        context: Rc<ProcessTokenContext>,
    ) -> Self {
        Self {
            factory_seed_vault,
            objective_seed_vault,
            bank_registry,
            wallet_registry,
            item_vault,
            context,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlantFacilityFactory {}

pub(crate) struct PlantFacilityFactoryDynSeed;

enum PlantFacilityRequest {
    Interact,
}

struct Console<'a> {
    id: ModuleId,
    requests: Vec<PlantFacilityRequest>,
    state: &'a mut PlantFacilityState,
    storage: &'a mut ItemStorage,
}

impl ModuleConsole for Console<'_> {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn type_id(&self) -> ModuleTypeId {
        crate::modules::fabricator::TYPE_ID.into()
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

    fn interact(&mut self) -> bool {
        let is_recipe_valid = |state: &PlantFacilityState| match state {
            PlantFacilityState::Idle => false,
            PlantFacilityState::Producing { recipe_index, .. } => *recipe_index < RECIPES.len(),
        };

        if !is_recipe_valid(self.state) {
            return false;
        }

        self.requests.push(PlantFacilityRequest::Interact);
        true
    }

    fn in_progress(&self) -> bool {
        match self.state {
            PlantFacilityState::Idle => false,
            PlantFacilityState::Producing { .. } => true,
        }
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        Some(self)
    }

    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole> {
        Some(self)
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

    fn trading_admin_console(&self) -> Option<&dyn AdminTradingConsole> {
        todo!()
    }

    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn AdminTradingConsole> {
        None
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![self.storage]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        match role {
            StorageRole::Input => vec![],
            StorageRole::Output => vec![self.storage],
            StorageRole::NoRole => vec![],
        }
    }

    fn storages_by_role_mut(&mut self, role: StorageRole) -> Vec<&mut ItemStorage> {
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
}

impl<'a> CraftingConsole for Console<'a> {
    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize> {
        todo!()
    }

    fn recipe_by_output_primary_capability(&self, capability: ModuleCapability) -> Option<usize> {
        todo!()
    }

    fn recipe_by_output_module(&self, type_id: ModuleTypeId) -> Option<usize> {
        todo!()
    }

    fn recipe_by_output_item(&self, item: ItemId) -> Option<usize> {
        todo!()
    }

    fn recipe_by_hash(&self, hash: ItemRecipeHash) -> Option<usize> {
        todo!()
    }

    fn recipe_by_output_hash(&self, hash: OutputItemRecipeHash) -> Option<usize> {
        RECIPES.iter().position(|r| r.hash() == hash)
    }

    fn recipe_by_input_hash(&self, hash: InputItemRecipeHash) -> Option<usize> {
        todo!()
    }

    fn recipe_output_description(&self, index: usize) -> &dyn ModuleFactoryOutputDescription {
        todo!()
    }

    fn recipe_item_output(&self, index: usize) -> Option<OutputItemRecipe> {
        RECIPES.get(index).cloned()
    }

    fn recipe_item_input(&self, index: usize) -> Option<InputItemRecipe> {
        todo!()
    }

    fn item_recipe(&self, index: usize) -> Option<ItemRecipe> {
        todo!()
    }

    fn has_resources_for_recipe(&self, index: usize) -> bool {
        true
    }

    fn active_recipe(&self) -> Option<usize> {
        todo!()
    }

    fn start(&mut self, index: usize, deploy: bool) -> Option<ProcessToken> {
        let (token, token_mut) = ProcessTokenMut::new();
        *self.state = PlantFacilityState::Producing {
            recipe_index: index,
            process_token: token_mut,
        };
        Some(token)
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        todo!()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        todo!()
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        RECIPES.as_ref()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }
}

impl Module for PlantFacility {
    fn id(&self) -> ModuleId {
        self.id.clone()
    }

    fn package_id(&self) -> PackageId {
        CORE_PACKAGE_ID.into()
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
        let mut console = Console {
            id: self.id,
            requests: vec![],
            state: &mut self.state,
            storage: &mut self.storage,
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
                PlantFacilityRequest::Interact => match &self.state {
                    PlantFacilityState::Idle => todo!(),
                    PlantFacilityState::Producing {
                        recipe_index,
                        process_token,
                    } => {
                        let ok = self
                            .storage
                            .try_insert_output(RECIPES[*recipe_index].clone());
                        assert!(ok);
                        self.state = PlantFacilityState::Idle;
                    }
                },
            }
        }
    }

    fn active(&self) -> bool {
        match &self.state {
            PlantFacilityState::Idle => false,
            PlantFacilityState::Producing { .. } => true,
        }
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        todo!()
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        &[]
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        RECIPES.as_ref()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        if self
            .operator
            .as_ref()
            .map(|p| p.id() == id)
            .unwrap_or(false)
        {
            self.operator.take()
        } else {
            None
        }
    }

    fn insert_person(&mut self, person: Person) -> bool {
        if self.operator.is_none() {
            self.operator = Some(person);
            true
        } else {
            false
        }
    }

    fn free_person_slots_count(&self) -> usize {
        const CAPACITY: usize = 1;
        CAPACITY - self.operator.iter().len()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        self.operator
            .as_ref()
            .map(|p| p.id() == id)
            .unwrap_or(false)
    }

    fn persons(&self) -> &[Person] {
        self.operator
            .as_ref()
            .map(std::slice::from_ref)
            .unwrap_or(&[])
    }

    fn persons_mut(&mut self) -> &mut [Person] {
        todo!()
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![&self.storage]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        match role {
            StorageRole::Input => vec![],
            StorageRole::Output => vec![&self.storage],
            StorageRole::NoRole => vec![],
        }
    }

    fn storages_by_role_mut(&mut self, role: StorageRole) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn safes(&self) -> &[ItemSafe] {
        &[]
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        &[]
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        &[]
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }

    fn docking_connectors(&self) -> &[DockingConnector] {
        &[]
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        None
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

impl ModuleFactory for PlantFacilityFactory {
    fn create(&self, item_vault: Rc<ItemVault>, recipe: &InputItemRecipe) -> Box<dyn Module> {
        PlantFacility::new(ItemStorage::new(item_vault, ITEM_STORAGE_CAPACITY))
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
        TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| Box::new(e) as Box<dyn Error>)
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
        let obj: PlantFacility = from_intermediate_seed(
            PlantFacilitySeed::new(
                &self.factory_seed_vault,
                &self.objective_seed_vault,
                &self.bank_registry,
                &self.wallet_registry,
                self.item_vault.clone(),
                &self.context,
            ),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
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
