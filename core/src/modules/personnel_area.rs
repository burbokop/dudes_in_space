use crate::CORE_PACKAGE_ID;
use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut};
use dudes_in_space_api::modules::{
    AssemblyRecipe, Module, ModuleCapability, ModuleId, PackageId, VesselPersonInterface,
};
use dudes_in_space_api::{Person, PersonId, Recipe};
use dyn_serde::{DynDeserializeSeed, DynSerialize};
use serde_intermediate::{from_intermediate, to_intermediate, Intermediate};
use std::error::Error;
use serde::{Deserialize, Serialize};

static TYPE_ID: &str = "PersonnelArea";

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonnelArea {
    personnel: Vec<Person>,
    id: ModuleId,
}

impl PersonnelArea {
    pub fn new(personnel: Vec<Person>) -> Box<Self> {
        Box::new(Self { id: PersonId::new_v4(), personnel })
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

    fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        for person in &mut self.personnel {
            person.proceed(v)
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
        self.personnel.iter().position(|x|x.id() == id).map(|x| self.personnel.remove(x))
    }

    fn insert_person(&mut self, person: Person) -> bool {
        todo!()
    }

    fn can_insert_person(&self) -> bool {
        todo!()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        self.personnel.iter().find(|p|(*p).id() == id).is_some()
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

pub struct PersonnelAreaDynSeed;

impl DynDeserializeSeed<dyn Module> for PersonnelAreaDynSeed {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: PersonnelArea = from_intermediate(&intermediate)
            .map_err(|e| e.to_string())?;

        Ok(Box::new(obj))
    }
}
