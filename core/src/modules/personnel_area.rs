use crate::CORE_PACKAGE_ID;
use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut};
use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::module::{
    DefaultModuleConsole, Module, ModuleCapability, ModuleId, ModuleStorage, PackageId,
    ProcessTokenContext,
};
use dudes_in_space_api::person::{Person, PersonId};
use dudes_in_space_api::recipe::{AssemblyRecipe, Recipe};
use dudes_in_space_api::vessel::VesselModuleInterface;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;

static TYPE_ID: &str = "PersonnelArea";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PersonnelArea {
    personnel: Vec<Person>,
    id: ModuleId,
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
    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        CORE_PACKAGE_ID.to_string()
    }

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        process_token_context: &ProcessTokenContext,
    ) {
        let mut person_interface = DefaultModuleConsole::new(self.id);
        for person in &mut self.personnel {
            person.proceed(
                &mut person_interface,
                this_vessel.console(),
                process_token_context,
            )
        }
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        &[]
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

    fn storages(&mut self) -> &mut [ItemStorage] {
        todo!()
    }

    fn module_storages(&mut self) -> &mut [ModuleStorage] {
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

pub(crate) struct PersonnelAreaDynSeed;

impl DynDeserializeSeed<dyn Module> for PersonnelAreaDynSeed {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: PersonnelArea = from_intermediate(&intermediate).map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}
