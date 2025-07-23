use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut};
use dudes_in_space_api::modules::{AssemblyRecipe, AssemblyRecipeSeed, Module, ModuleCapability, ModuleFactory, ModuleId, ModulePersonInterface, ModuleStorage, PackageId, VesselModuleInterface, VesselPersonInterface, WorkerControlPanel};
use dudes_in_space_api::{Item, ItemStorage, Person, PersonId, Recipe};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, VecSeed, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::ops::Deref;
use std::rc::Rc;
use crate::modules::assembler::AssemblerState::Idle;

static TYPE_ID: &str = "Assembler";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::Crafting, ModuleCapability::ItemStorage];

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tp")]
enum AssemblerState {
    Idle,
    Assembling {
        recipe_index: usize,
        deploy: bool,
    },
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::assembler::AssemblerSeed::<'v>)]
pub struct Assembler {
    id: ModuleId,
    #[deserialize_seed_xxx(seed = self.seed.assembly_recipe_seq_seed)]
    recipes: Vec<AssemblyRecipe>,
    state: AssemblerState,
    storage: ItemStorage,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    operator: Option<Person>,
}

#[derive(Clone)]
pub struct AssemblerSeed<'v> {
    assembly_recipe_seq_seed: VecSeed<AssemblyRecipeSeed<'v>>,
}

impl<'v> AssemblerSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn ModuleFactory>) -> Self {
        Self {
            assembly_recipe_seq_seed: VecSeed::new(AssemblyRecipeSeed::new(vault)),
        }
    }
}

impl WorkerControlPanel for Assembler {}

impl Assembler {
    pub fn new(recipes: Vec<AssemblyRecipe>) -> Box<Self> {
        Box::new(Self {
            id: ModuleId::new_v4(),
            recipes,
            state: Idle,
            storage: Default::default(),
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

struct AssemblerPersonInterface<'a> {
    id: PersonId,
    recipes: &'a [AssemblyRecipe],
    requests: Vec<AssemblerRequest>,
    state: &'a mut AssemblerState,
    storage: &'a mut ItemStorage,
}

impl<'a> ModulePersonInterface for AssemblerPersonInterface<'a> {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize> {
        self.recipes
            .iter()
            .position(|recipe| recipe.output_capabilities().contains(&capability))
    }

    fn recipe_output_capabilities(&self, index: usize) -> &[ModuleCapability] {
        self.recipes[index].output_capabilities()
    }

    fn has_resources_for_recipe(&self, index: usize) -> bool {
        self.storage
            .contains_for_input(self.recipes[index].input().clone())
    }

    fn active_recipe(&self) -> Option<usize> {
        match self.state.deref() {
            Idle => None,
            AssemblerState::Assembling { recipe_index, .. } => Some(*recipe_index),
        }
    }

    fn start_assembly(&mut self, index: usize, deploy: bool) -> bool {
        *self.state = AssemblerState::Assembling { recipe_index: index, deploy };
        true
    }

    fn interact(&mut self) -> bool {
        let is_recipe_valid = |state: &AssemblerState| {
            match state {
                Idle => false,
                AssemblerState::Assembling { recipe_index,.. } => *recipe_index < self.recipes.len(),
            }
        };

        if !is_recipe_valid(self.state)
        {
            return false;
        }

        self.requests.push(AssemblerRequest::Interact);
        true
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
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

    fn proceed(&mut self, this_vessel: &dyn VesselModuleInterface) {
        let mut person_interface = AssemblerPersonInterface {
            id: self.id,
            recipes: &self.recipes,
            requests: vec![],
            state: &mut self.state,
            storage: &mut self.storage,
        };

        if let Some(operator) = &mut self.operator {
            operator.proceed(&mut person_interface, this_vessel.vessel_person_interface())
        }

        for request in std::mem::take(&mut person_interface.requests) {
            match request {
                AssemblerRequest::SetRecipe(_) => {
                    todo!()
                }
                AssemblerRequest::Interact => {
                    match self.state {
                        Idle => todo!(),
                        AssemblerState::Assembling { recipe_index, deploy } => {
                            let active_recipe = &self.recipes[recipe_index];

                            let ok = self.storage.try_consume(active_recipe.input().clone());
                            assert!(ok);

                            if deploy {
                                this_vessel.add_module(active_recipe.create());
                                self.state = Idle;
                            } else {
                                let mut storage_modules = this_vessel.vessel_person_interface().modules_with_cap(ModuleCapability::ModuleStorage);
                                assert!(!storage_modules.is_empty());
                                assert!(!storage_modules[0].module_storages().is_empty());
                                let storage = &mut storage_modules[0].module_storages()[0];
                                assert!(storage.has_space());
                                let ok = storage.add(active_recipe.create());
                                assert!(ok);
                                self.state = Idle;
                            }
                        }
                    }
                }
            }
        }
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &self.recipes
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        todo!()
    }

    fn insert_person(&mut self, person: Person) -> bool {
        if self.operator.is_none() {
            self.operator = Some(person);
            true
        } else {
            false
        }
    }

    fn can_insert_person(&self) -> bool {
        self.operator.is_none()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        self.operator
            .as_ref()
            .map(|p| p.id() == id)
            .unwrap_or(false)
    }

    fn storages(&mut self) -> &mut [ItemStorage] {
        todo!()
    }

    fn module_storages(&mut self) -> &mut [ModuleStorage] {
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
    seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
}

impl AssemblerDynSeed {
    pub fn new(seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>) -> Self {
        Self { seed_vault }
    }
}

impl DynDeserializeSeed<dyn Module> for AssemblerDynSeed {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate, _: &DynDeserializeSeedVault<dyn Module>) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Assembler =
            from_intermediate_seed(AssemblerSeed::new(&self.seed_vault), &intermediate)
                .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}
