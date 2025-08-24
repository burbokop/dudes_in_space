use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::BankRegistry;
use dudes_in_space_api::item::{ItemSafe, ItemStorage};
use dudes_in_space_api::module::{
    CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, ModuleStorageSeed, ModuleTypeId, PackageId, ProcessToken, ProcessTokenContext,
    ProcessTokenMut, ProcessTokenMutSeed, TradingAdminConsole, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{
    DockingClamp, DockingClampSeed, DockingConnector, Vessel, VesselModuleInterface,
};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;
use crate::CORE_PACKAGE_ID;

static TYPE_ID: &str = "Dockyard";
static FACTORY_TYPE_ID: &str = "DockyardFactory";
static DOCKING_CLAMP_COMPAT_TYPE: usize = 0;
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Dockyard,
    ModuleCapability::ModuleStorage,
    ModuleCapability::PersonnelRoom,
    ModuleCapability::DockingClamp,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::Dockyard];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::dockyard::DockyardStateSeed::<'context>)]
#[serde(tag = "tp")]
enum DockyardState {
    Idle,
    #[deserialize_seed_xxx(seeds = [(process_token, self.seed.seed.process_token_seed)])]
    Building {
        modules: BTreeSet<ModuleId>,
        process_token: ProcessTokenMut,
    },
}

#[derive(Clone)]
struct DockyardStateSeed<'context> {
    process_token_seed: ProcessTokenMutSeed<'context>,
}

impl<'context> DockyardStateSeed<'context> {
    pub fn new(context: &'context ProcessTokenContext) -> Self {
        Self {
            process_token_seed: ProcessTokenMutSeed::new(context),
        }
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::dockyard::DockyardSeed::<'v,'b, 'context>)]
pub struct Dockyard {
    id: ModuleId,
    #[deserialize_seed_xxx(seed = self.seed.state_seed)]
    state: DockyardState,
    #[deserialize_seed_xxx(seed = self.seed.module_storage_seed)]
    module_storage: ModuleStorage,
    #[deserialize_seed_xxx(seed = self.seed.docking_clamp_seed)]
    docking_clamp: DockingClamp,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}

impl Dockyard {
    fn new(compat_type: usize) -> Self {
        Self {
            id: ModuleId::new_v4(),
            state: DockyardState::Idle,
            module_storage: Default::default(),
            docking_clamp: DockingClamp::new(compat_type),
            operator: None,
        }
    }
}

#[derive(Clone)]
struct DockyardSeed<'v, 'b, 'context> {
    module_storage_seed: ModuleStorageSeed<'v>,
    docking_clamp_seed: DockingClampSeed<'v>,
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'b>>,
    state_seed: DockyardStateSeed<'context>,
}

impl<'v, 'b, 'context> DockyardSeed<'v, 'b, 'context> {
    fn new(
        module_vault: &'v DynDeserializeSeedVault<dyn Module>,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'b BankRegistry,
        context: &'context ProcessTokenContext,
    ) -> Self {
        Self {
            module_storage_seed: ModuleStorageSeed::new(module_vault),
            docking_clamp_seed: DockingClampSeed::new(module_vault),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(objective_vault, bank_registry)),
            state_seed: DockyardStateSeed::new(context),
        }
    }
}

impl DynSerialize for Dockyard {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

enum DockyardRequest {
    SetRecipe(usize),
    Interact,
}

struct Console<'a> {
    id: ModuleId,
    requests: Vec<DockyardRequest>,
    state: &'a mut DockyardState,
    module_storage: &'a mut ModuleStorage,
    docking_clamp: &'a mut DockingClamp,
}

impl<'a> ModuleConsole for Console<'a> {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn type_id(&self) -> ModuleTypeId {
        todo!()
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
        let is_state_valid = |state: &DockyardState| match state {
            DockyardState::Idle => false,
            DockyardState::Building {
                modules,
                process_token,
            } => !modules.is_empty(),
        };

        if !is_state_valid(self.state) {
            return false;
        }

        if self.docking_clamp.is_docked() {
            return false;
        }

        self.requests.push(DockyardRequest::Interact);
        true
    }

    fn in_progress(&self) -> bool {
        match self.state {
            DockyardState::Idle => false,
            DockyardState::Building { .. } => true,
        }
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        None
    }

    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole> {
        todo!()
    }

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole> {
        todo!()
    }

    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole> {
        Some(self)
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

    fn safes(&self) -> &[ItemSafe] {
        todo!()
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        std::slice::from_ref(self.module_storage)
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        std::slice::from_ref(self.docking_clamp)
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        std::slice::from_mut(self.docking_clamp)
    }
}

impl<'a> DockyardConsole for Console<'a> {
    fn start(&mut self, modules: BTreeSet<ModuleId>) -> Option<ProcessToken> {
        let (token, token_mut) = ProcessTokenMut::new();
        *self.state = DockyardState::Building {
            modules,
            process_token: token_mut,
        };
        Some(token)
    }
}

impl Module for Dockyard {
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
        let rng = &mut rng();

        let mut console = Console {
            id: self.id,
            requests: vec![],
            state: &mut self.state,
            module_storage: &mut self.module_storage,
            docking_clamp: &mut self.docking_clamp,
        };

        if let Some(operator) = &mut self.operator {
            operator.proceed(
                rng,
                &mut console,
                this_vessel.console(),
                environment_context,
                decider_vault,
                logger,
            )
        }

        for request in std::mem::take(&mut console.requests) {
            match request {
                DockyardRequest::SetRecipe(_) => {
                    todo!()
                }
                DockyardRequest::Interact => match &mut self.state {
                    DockyardState::Idle => todo!(),
                    DockyardState::Building {
                        modules,
                        process_token,
                    } => {
                        if self.docking_clamp.is_empty() {
                            let modules = self.module_storage.try_take(modules.iter()).unwrap();

                            self.docking_clamp
                                .dock(Vessel::new(
                                    format!("SS-{}", rng.random_range(1000..9999)),
                                    this_vessel.console().owner(),
                                    (0., 0.).into(),
                                    modules,
                                ))
                                .unwrap();
                            process_token
                                .mark_completed(environment_context.process_token_context());
                            self.state = DockyardState::Idle;
                        } else {
                            todo!()
                        }
                    }
                },
            }
        }

        self.docking_clamp
            .proceed(environment_context, decider_vault, logger);
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        if let Some(operator) = &self.operator {
            operator.collect_status(collector);
        }
        if let Some(connection) = &self.docking_clamp.connection() {
            connection.vessel.collect_status(collector);
        }
        collector.exit_module();
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
        match self.operator.as_ref() {
            None => &[],
            Some(person) => std::slice::from_ref(person),
        }
    }

    fn storages(&self) -> &[ItemStorage] {
        todo!()
    }

    fn storages_mut(&mut self) -> &mut [ItemStorage] {
        todo!()
    }

    fn safes(&self) -> &[ItemSafe] {
        todo!()
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        std::slice::from_ref(&self.module_storage)
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        std::slice::from_mut(&mut self.module_storage)
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        std::slice::from_ref(&self.docking_clamp)
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DockyardFactory {}

impl DynSerialize for DockyardFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl ModuleFactory for DockyardFactory {
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(Dockyard::new(DOCKING_CLAMP_COMPAT_TYPE))
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for DockyardFactory {
    fn type_id(&self) -> ModuleTypeId {
        todo!()
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

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        &[]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }
}

pub(crate) struct DockyardDynSeed {
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    context: Rc<ProcessTokenContext>,
}

impl DockyardDynSeed {
    pub fn new(
        objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
        context: Rc<ProcessTokenContext>,
    ) -> Self {
        Self {
            objective_seed_vault,
            bank_registry,
            context,
        }
    }
}

impl DynDeserializeSeed<dyn Module> for DockyardDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Dockyard = from_intermediate_seed(
            DockyardSeed::new(
                this_vault,
                &self.objective_seed_vault,
                &self.bank_registry,
                &self.context,
            ),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}

pub(crate) struct DockyardFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for DockyardFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<DockyardFactory> =
            serde_intermediate::from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
