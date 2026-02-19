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
    AssemblyRecipe, InputItemRecipe, InputItemRecipeHash, ItemRecipe, ItemRecipeHash,
    ModuleFactory, ModuleFactoryOutputDescription, OutputItemRecipe, OutputItemRecipeHash,
};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::clone::Clone;
use std::convert::Into;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, LazyLock};

pub(super) static TYPE_ID: &str = "Fabricator";
static FACTORY_TYPE_ID: &str = "FabricatorFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::ItemCrafting,
    ModuleCapability::ItemConsumption,
    ModuleCapability::ItemProduction,
    ModuleCapability::ItemStorage,
    ModuleCapability::PersonnelRoom,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemCrafting];
static ITEM_STORAGE_CAPACITY: ItemVolume = M3(100);

pub(crate) static RECIPES: LazyLock<[ItemRecipe; 8]> = LazyLock::new(|| {
    [
        ItemRecipe {
            input: [("ice".into(), 10)].into(),
            output: [("water".into(), 10), ("gangue".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("biomass".into(), 10)].into(),
            output: [("carbon".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("carbon".into(), 10)].into(),
            output: [("plastic".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("silicon_ore".into(), 10)].into(),
            output: [("silicon".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("iron_ore".into(), 10), ("carbon".into(), 10)].into(),
            output: [("steel".into(), 10), ("gangue".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("rare_earth_ore".into(), 10)].into(),
            output: [("rare_earth_alloys".into(), 10), ("gangue".into(), 10)].into(),
        },
        ItemRecipe {
            input: [
                ("silicon".into(), 10),
                ("rare_earth_alloys".into(), 10),
                ("plastic".into(), 10),
            ]
            .into(),
            output: [("microelectronics".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("steel".into(), 10)].into(),
            output: [("heat_cell".into(), 10)].into(),
        },
    ]
});

static INPUT_RECIPES: LazyLock<[InputItemRecipe; 8]> =
    LazyLock::new(|| RECIPES.clone().map(|x| x.input));
static OUTPUT_RECIPES: LazyLock<[OutputItemRecipe; 8]> =
    LazyLock::new(|| RECIPES.clone().map(|x| x.output));

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
#[deserialize_seed_xxx(seed = crate::modules::fabricator::FabricatorSeed::<'v,'a,'b, 'context>)]
struct Fabricator {
    id: ModuleId,
    // recipes: Vec<ItemRecipe>,
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

struct FabricatorSeed<'v, 'a, 'b, 'context> {
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'a, 'b>>,
    item_storage_seed: ItemStorageSeed,
    state_seed: FabricatorStateSeed<'context>,
}

impl<'v, 'a, 'b, 'context> FabricatorSeed<'v, 'a, 'b, 'context> {
    fn new(
        objective_seed_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'a BankRegistry,
        wallet_registry: &'b WalletRegistry,
        item_vault: Rc<ItemVault>,
        context: &'context ProcessTokenContext,
    ) -> Self {
        Self {
            person_seed: TaggedOptionSeed::new(PersonSeed::new(
                objective_seed_vault,
                bank_registry,
                wallet_registry,
            )),
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
    requests: Vec<FabricatorRequest>,
    state: &'a mut FabricatorState,
    input_storage: &'a mut ItemStorage,
    output_storage: &'a mut ItemStorage,
}

impl ModuleConsole for Console<'_> {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn type_id(&self) -> ModuleTypeId {
        TYPE_ID.into()
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
        todo!()
    }

    fn in_progress(&self) -> bool {
        todo!()
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
        todo!()
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![self.input_storage, self.output_storage]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        match role {
            StorageRole::Input => vec![self.input_storage],
            StorageRole::Output => vec![self.output_storage],
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
        RECIPES.iter().position(|r| r.hash() == hash)
    }

    fn recipe_by_output_hash(&self, hash: OutputItemRecipeHash) -> Option<usize> {
        RECIPES.iter().position(|x| x.output.hash() == hash)
    }

    fn recipe_by_input_hash(&self, hash: InputItemRecipeHash) -> Option<usize> {
        todo!()
    }

    fn recipe_output_description(&self, index: usize) -> &dyn ModuleFactoryOutputDescription {
        todo!()
    }

    fn recipe_item_output(&self, index: usize) -> Option<OutputItemRecipe> {
        todo!()
    }

    fn recipe_item_input(&self, index: usize) -> Option<InputItemRecipe> {
        todo!()
    }

    fn item_recipe(&self, index: usize) -> Option<ItemRecipe> {
        RECIPES.get(index).cloned()
    }

    fn has_resources_for_recipe(&self, index: usize) -> bool {
        assert!(index < RECIPES.len());
        self.input_storage
            .contains_for_input(RECIPES[index].input.clone())
    }

    fn active_recipe(&self) -> Option<usize> {
        todo!()
    }

    fn start(&mut self, index: usize, deploy: bool) -> Option<ProcessToken> {
        todo!()
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        RECIPES.as_ref()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        INPUT_RECIPES.as_ref()
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        OUTPUT_RECIPES.as_ref()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }
}

impl Module for Fabricator {
    fn id(&self) -> ModuleId {
        self.id
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
                        RECIPES[*recipe_index]
                            .craft(&mut self.input_storage, &mut self.output_storage)
                            .unwrap();
                        self.state = FabricatorState::Idle;
                    }
                },
            }
        }
    }

    fn active(&self) -> bool {
        match &self.state {
            FabricatorState::Idle => false,
            FabricatorState::Fabricating { .. } => true,
        }
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        if let Some(operator) = &self.operator {
            operator.collect_status(collector);
        }
        collector.exit_module();
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        RECIPES.as_ref()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        INPUT_RECIPES.as_ref()
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        OUTPUT_RECIPES.as_ref()
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
        self.operator
            .as_mut()
            .map(std::slice::from_mut)
            .unwrap_or(&mut [])
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![&self.input_storage, &self.output_storage]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        match role {
            StorageRole::Input => vec![&self.input_storage],
            StorageRole::Output => vec![&self.output_storage],
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

pub(crate) struct FabricatorDynSeed {
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    wallet_registry: Rc<WalletRegistry>,
    item_vault: Rc<ItemVault>,
    context: Rc<ProcessTokenContext>,
}

impl FabricatorDynSeed {
    pub(crate) fn new(
        objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
        wallet_registry: Rc<WalletRegistry>,
        item_vault: Rc<ItemVault>,
        context: Rc<ProcessTokenContext>,
    ) -> Self {
        Self {
            objective_seed_vault,
            bank_registry,
            wallet_registry,
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
            FabricatorSeed::new(
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FabricatorFactory {}
struct FabricatorFactorySeed {}

impl DynSerialize for FabricatorFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl ModuleFactory for FabricatorFactory {
    fn create(&self, item_vault: Rc<ItemVault>, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(Fabricator {
            id: ModuleId::new_v4(),
            state: FabricatorState::Idle,
            input_storage: ItemStorage::new(item_vault.clone(), ITEM_STORAGE_CAPACITY),
            output_storage: ItemStorage::new(item_vault, ITEM_STORAGE_CAPACITY),
            operator: None,
        })
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for FabricatorFactory {
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
        RECIPES.as_ref()
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        OUTPUT_RECIPES.as_ref()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        INPUT_RECIPES.as_ref()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
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
            from_intermediate(&intermediate).map_err(|e| e.to_string())?;

        let xxx: Arc<FabricatorFactory> = r.into();

        let yyy: Arc<dyn ModuleFactory> = xxx;

        todo!()
        // Ok(r)
    }
}
