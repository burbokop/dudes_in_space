use crate::CORE_PACKAGE_ID;
use crate::modules::{CoreModule, DockyardDynSeed, ModuleVisitor, ModuleVisitorMut};
use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::module::{DefaultModuleConsole, Module, ModuleCapability, ModuleConsole, ModuleId, ModuleStorage, ModuleStorageSeed, PackageId, ProcessTokenContext, TradingConsole};
use dudes_in_space_api::person::{DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed};
use dudes_in_space_api::recipe::{AssemblyRecipe, Recipe};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingClampSeed, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, VecSeed, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::rc::Rc;

static TYPE_ID: &str = "PersonnelArea";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::PersonnelRoom];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::personnel_area::PersonnelAreaSeed::<'v>)]
pub(crate) struct PersonnelArea {
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    personnel: Vec<Person>,
    id: ModuleId,
}

#[derive(Clone)]
struct PersonnelAreaSeed<'v> {
    person_seed: VecSeed<PersonSeed<'v>>,
}

impl<'v> PersonnelAreaSeed<'v> {
    fn new(objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>) -> Self {
        Self {
            person_seed: VecSeed::new(PersonSeed::new(objective_vault)),
        }
    }
}

impl PersonnelArea {
    pub fn new(personnel: Vec<Person>) -> Box<Self> {
        Box::new(Self {
            id: PersonId::new_v4(),
            personnel,
        })
    }
}

impl DynSerialize for PersonnelArea {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl Module for PersonnelArea {
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

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        let mut person_interface = DefaultModuleConsole::new(self.id);
        for person in &mut self.personnel {
            person.proceed(
                &mut rng(),
                &mut person_interface,
                this_vessel.console(),
                process_token_context,
                decider_vault,
                logger,
            )
        }
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        todo!()
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        self.personnel
            .iter()
            .position(|x| x.id() == id)
            .map(|x| self.personnel.remove(x))
    }

    fn insert_person(&mut self, person: Person) -> bool {
        todo!()
    }

    fn can_insert_person(&self) -> bool {
        todo!()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        self.personnel.iter().find(|p| (*p).id() == id).is_some()
    }

    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

impl CoreModule for PersonnelArea {
    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
        v.visit_personnel_area(self)
    }

    fn accept_visitor_mut(&mut self, v: &dyn ModuleVisitorMut<Result = ()>) -> Option<()> {
        v.visit_personnel_area(self)
    }
}

pub(crate) struct PersonnelAreaDynSeed {
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
}

impl PersonnelAreaDynSeed {
    pub fn new(objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>) -> Self {
        Self {
            objective_seed_vault,
        }
    }
}

impl DynDeserializeSeed<dyn Module> for PersonnelAreaDynSeed {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: PersonnelArea = from_intermediate_seed(
            PersonnelAreaSeed::new(&self.objective_seed_vault),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}
