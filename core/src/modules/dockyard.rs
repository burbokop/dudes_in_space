use dudes_in_space_api::modules::{AssemblyRecipe, Module, ModuleCapability, ModuleFactory, ModuleId, ModulePersonInterface, ModuleStorage, ModuleStorageSeed, ModuleTypeId, PackageId, VesselModuleInterface};
use dudes_in_space_api::{InputRecipe, ItemStorage, Person, PersonId, Recipe};
use dyn_serde::{from_intermediate_seed, DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde_intermediate::{to_intermediate, Intermediate};
use std::error::Error;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use dyn_serde_macro::DeserializeSeedXXX;

static TYPE_ID: &str = "Dockyard";
static FACTORY_TYPE_ID: &str = "DockyardFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::Dockyard];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::dockyard::DockyardSeed::<'v>)]
pub struct Dockyard {
    id: ModuleId,
    #[deserialize_seed_xxx(seed = self.seed.module_storage_seed)]
    module_storage: ModuleStorage,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    operator: Option<Person>,
}

impl Dockyard {
    fn new() -> Self {
        Self {
            id: ModuleId::new_v4(),
            module_storage: Default::default(),
            operator: None,
        }
    }
}

#[derive(Clone)]
struct DockyardSeed<'v> {
    module_storage_seed: ModuleStorageSeed<'v>,
}

impl<'v> DockyardSeed<'v> {
    fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> Self {
        Self {
            module_storage_seed: ModuleStorageSeed::new(vault),
        }
    }
}

impl DynSerialize for Dockyard {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self)  .map_err(|e|e.into())
    }
}

enum DockyardRequest {
    SetRecipe(usize),
    Interact,
}

struct DockyardPersonInterface {
    id: PersonId,
    requests: Vec<DockyardRequest>,
}

impl ModulePersonInterface for DockyardPersonInterface {
    fn id(&self) -> ModuleId {
        todo!()
    }

    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize> {
        todo!()
    }

    fn has_resources_for_recipe(&self, index: usize) -> bool {
        todo!()
    }

    fn active_recipe(&self) -> Option<usize> {
        todo!()
    }

    fn start_assembly(&mut self, index: usize, deploy: bool) -> bool {
        todo!()
    }


    fn interact(&mut self) -> bool {
        todo!()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        todo!()
    }
}

impl Module for Dockyard {
    fn id(&self) -> ModuleId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn proceed(&mut self, this_vessel: &dyn VesselModuleInterface) {
        let mut person_interface = DockyardPersonInterface {
            id: self.id,
            requests: vec![],
        };

        if let Some(operator) = &mut self.operator {
            operator.proceed(&mut person_interface, this_vessel.vessel_person_interface())
        }

        for request in std::mem::take(&mut person_interface.requests) {
            match request {
                DockyardRequest::SetRecipe(_) => {
                    todo!()
                }
                DockyardRequest::Interact => {
                    todo!()
                }
            }
        }
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
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

    fn can_insert_person(&self) -> bool {
        todo!()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        todo!()
    }

    fn storages(&mut self) -> &mut [ItemStorage] {
        todo!()
    }

    fn module_storages(&mut self) -> &mut [ModuleStorage] {
        std::slice::from_mut(&mut self.module_storage)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DockyardFactory {}

impl DynSerialize for DockyardFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl ModuleFactory for DockyardFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, recipe: &InputRecipe) -> Box<dyn Module> {
        Box::new(Dockyard::new())
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }
}

pub(crate) struct DockyardDynSeed;

impl DynDeserializeSeed<dyn Module> for DockyardDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate, this_vault: &DynDeserializeSeedVault<dyn Module>) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Dockyard =
            from_intermediate_seed(DockyardSeed::new(this_vault), &intermediate)
                .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}

pub(crate) struct DockyardFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for DockyardFactoryDynSeed {
    fn type_id(&self) -> TypeId {
       FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate, this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<DockyardFactory> =
            serde_intermediate::from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
