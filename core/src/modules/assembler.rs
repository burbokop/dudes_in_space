use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut};
use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::module::{
    AssemblyConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, PackageId, ProcessToken, ProcessTokenContext, ProcessTokenMut,
    ProcessTokenMutSeed, TradingAdminConsole, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, AssemblyRecipeSeed, ModuleFactory, Recipe};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, VecSeed, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::ops::Deref;
use std::rc::Rc;

static TYPE_ID: &str = "Assembler";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Crafting,
    ModuleCapability::ItemStorage,
    ModuleCapability::PersonnelRoom,
];

static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::Crafting];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::assembler::AssemblerStateSeed::<'context>)]
#[serde(tag = "tp")]
pub enum AssemblerState {
    Idle,
    #[deserialize_seed_xxx(seeds = [(process_token, self.seed.seed.process_token_seed)])]
    Assembling {
        recipe_index: usize,
        deploy: bool,
        process_token: ProcessTokenMut,
    },
}

#[derive(Clone)]
struct AssemblerStateSeed<'context> {
    process_token_seed: ProcessTokenMutSeed<'context>,
}

impl<'context> AssemblerStateSeed<'context> {
    pub fn new(context: &'context ProcessTokenContext) -> Self {
        Self {
            process_token_seed: ProcessTokenMutSeed::new(context),
        }
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::assembler::AssemblerSeed::<'v, 'context>)]
pub struct Assembler {
    id: ModuleId,
    #[deserialize_seed_xxx(seed = self.seed.recipe_seq_seed)]
    recipes: Vec<AssemblyRecipe>,
    #[deserialize_seed_xxx(seed = self.seed.state_seed)]
    state: AssemblerState,
    storage: ItemStorage,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}

#[derive(Clone)]
pub struct AssemblerSeed<'v, 'context> {
    recipe_seq_seed: VecSeed<AssemblyRecipeSeed<'v>>,
    person_seed: TaggedOptionSeed<PersonSeed<'v>>,
    state_seed: AssemblerStateSeed<'context>,
}

impl<'v, 'context> AssemblerSeed<'v, 'context> {
    pub fn new(
        module_factory_vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        context: &'context ProcessTokenContext,
    ) -> Self {
        Self {
            recipe_seq_seed: VecSeed::new(AssemblyRecipeSeed::new(module_factory_vault)),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(objective_vault)),
            state_seed: AssemblerStateSeed::new(context),
        }
    }
}

impl Assembler {
    pub fn new(recipes: Vec<AssemblyRecipe>, storage: ItemStorage) -> Box<Self> {
        Box::new(Self {
            id: ModuleId::new_v4(),
            recipes,
            state: AssemblerState::Idle,
            storage,
            operator: None,
        })
    }

    pub fn add_recipe(&mut self, recipe: AssemblyRecipe) {
        self.recipes.push(recipe);
    }
}

impl DynSerialize for Assembler {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

enum AssemblerRequest {
    SetRecipe(usize),
    Interact,
}

struct Console<'a> {
    id: ModuleId,
    recipes: &'a [AssemblyRecipe],
    requests: Vec<AssemblerRequest>,
    state: &'a mut AssemblerState,
    storage: &'a mut ItemStorage,
}

impl<'a> ModuleConsole for Console<'a> {
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
        todo!()
    }

    fn interact(&mut self) -> bool {
        let is_recipe_valid = |state: &AssemblerState| match state {
            AssemblerState::Idle => false,
            AssemblerState::Assembling { recipe_index, .. } => *recipe_index < self.recipes.len(),
        };

        if !is_recipe_valid(self.state) {
            return false;
        }

        self.requests.push(AssemblerRequest::Interact);
        true
    }

    fn in_progress(&self) -> bool {
        match self.state {
            AssemblerState::Idle => false,
            AssemblerState::Assembling { .. } => true,
        }
    }

    fn assembly_console(&self) -> Option<&dyn AssemblyConsole> {
        Some(self)
    }

    fn assembly_console_mut(&mut self) -> Option<&mut dyn AssemblyConsole> {
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
        &[]
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        &mut []
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }
}

impl<'a> AssemblyConsole for Console<'a> {
    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize> {
        self.recipes
            .iter()
            .position(|recipe| recipe.output_capabilities().contains(&capability))
    }

    fn recipe_by_output_primary_capability(&self, capability: ModuleCapability) -> Option<usize> {
        self.recipes
            .iter()
            .position(|recipe| recipe.output_primary_capabilities().contains(&capability))
    }

    fn recipe_output_capabilities(&self, index: usize) -> &[ModuleCapability] {
        self.recipes[index].output_capabilities()
    }

    fn recipe_output_primary_capabilities(&self, index: usize) -> &[ModuleCapability] {
        self.recipes[index].output_primary_capabilities()
    }

    fn has_resources_for_recipe(&self, index: usize) -> bool {
        self.storage
            .contains_for_input(self.recipes[index].input().clone())
    }

    fn active_recipe(&self) -> Option<usize> {
        match self.state.deref() {
            AssemblerState::Idle => None,
            AssemblerState::Assembling { recipe_index, .. } => Some(*recipe_index),
        }
    }

    fn start(&mut self, index: usize, deploy: bool) -> Option<ProcessToken> {
        let (token, token_mut) = ProcessTokenMut::new();

        *self.state = AssemblerState::Assembling {
            recipe_index: index,
            deploy,
            process_token: token_mut,
        };

        Some(token)
    }

    fn recipes(&self) -> &[AssemblyRecipe] {
        self.recipes
    }
}

impl Module for Assembler {
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
        let mut console = Console {
            id: self.id,
            recipes: &self.recipes,
            requests: vec![],
            state: &mut self.state,
            storage: &mut self.storage,
        };

        if let Some(operator) = &mut self.operator {
            operator.proceed(
                &mut rng(),
                &mut console,
                this_vessel.console(),
                process_token_context,
                decider_vault,
                logger,
            )
        }

        for request in std::mem::take(&mut console.requests) {
            match request {
                AssemblerRequest::SetRecipe(_) => {
                    todo!()
                }
                AssemblerRequest::Interact => match &self.state {
                    AssemblerState::Idle => todo!(),
                    AssemblerState::Assembling {
                        recipe_index,
                        deploy,
                        process_token,
                    } => {
                        let active_recipe = &self.recipes[*recipe_index];

                        let ok = self.storage.try_consume(active_recipe.input().clone());
                        assert!(ok);

                        if *deploy {
                            this_vessel.add_module(active_recipe.create());
                            self.state = AssemblerState::Idle;
                        } else {
                            let mut storage_modules = this_vessel
                                .console()
                                .modules_with_capability(ModuleCapability::ModuleStorage);
                            assert!(!storage_modules.is_empty());
                            assert!(!storage_modules[0].module_storages().is_empty());
                            let storage = &mut storage_modules[0].module_storages_mut()[0];
                            assert!(storage.has_space());
                            let ok = storage.add(active_recipe.create());
                            assert!(ok);
                            self.state = AssemblerState::Idle;
                        }
                    }
                },
            }
        }
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &self.recipes
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

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

impl CoreModule for Assembler {
    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
        v.visit_assembler(self)
    }

    fn accept_visitor_mut(&mut self, v: &dyn ModuleVisitorMut<Result = ()>) -> Option<()> {
        v.visit_assembler(self)
    }
}

pub(crate) struct AssemblerDynSeed {
    factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    context: Rc<ProcessTokenContext>,
}

impl AssemblerDynSeed {
    pub fn new(
        factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
        objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        context: Rc<ProcessTokenContext>,
    ) -> Self {
        Self {
            factory_seed_vault,
            objective_seed_vault,
            context,
        }
    }
}

impl DynDeserializeSeed<dyn Module> for AssemblerDynSeed {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        _: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Assembler = from_intermediate_seed(
            AssemblerSeed::new(
                &self.factory_seed_vault,
                &self.objective_seed_vault,
                &self.context,
            ),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}

#[cfg(test)]
mod tests {
    use rand::rng;
    use serde_intermediate::{to_intermediate, Intermediate};
    use dudes_in_space_api::module::{Module, ProcessTokenContext};
    use dudes_in_space_api::person::{DynObjective, Person};
    use dudes_in_space_api::recipe::ModuleFactory;
    use dyn_serde::{from_intermediate_seed, DynDeserializeSeedVault};
    use super::{Assembler, AssemblerSeed};

    #[test]
    fn serde() {
        let mut assembler = Assembler::new(vec![]);
        assert!(assembler.can_insert_person());
        assert!(assembler.insert_person(Person::random(&mut rng())));
        assert!(!assembler.can_insert_person());

        let intermediate = to_intermediate(&assembler).unwrap();
        let json = serde_json::to_string(&intermediate).unwrap();
        let parsed_intermediate: Intermediate = serde_json::from_str(&json).unwrap();
        
        let module_factory_vault = DynDeserializeSeedVault::<dyn ModuleFactory>::new();
        let objective_vault = DynDeserializeSeedVault::<dyn DynObjective>::new();
        let process_token_context = ProcessTokenContext::new();

        let parsed_assembler: Assembler =
            from_intermediate_seed(AssemblerSeed::new(&module_factory_vault, &objective_vault,&process_token_context), &parsed_intermediate).unwrap();
    }
}
