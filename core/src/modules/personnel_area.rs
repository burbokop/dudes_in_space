use serde_intermediate::Intermediate;
use std::error::Error;
use dudes_in_space_api::modules::{AssemblyRecipe, Module, ModuleCapability, PackageId, VesselPersonInterface};
use dudes_in_space_api::{Person, Recipe};
use dyn_serde::{DynDeserializeSeed, DynSerialize};
use crate::CORE_PACKAGE_ID;
use crate::modules::{CoreModule, ModuleVisitor, ModuleVisitorMut};

static TYPE_ID: &str = "PersonnelArea";

#[derive(Debug)]
pub struct PersonnelArea {
    personnel: Vec<Person>,
}

impl PersonnelArea {
    pub fn new(personnel: Vec<Person>) -> Box<Self> {
        Box::new(Self { personnel })
    }
}

impl DynSerialize for PersonnelArea {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        serde_intermediate::to_intermediate(&self.personnel)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl Module for PersonnelArea {
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
}

impl CoreModule for PersonnelArea {
    fn accept_visitor(& self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
        v.visit_personnel_area(self)
    }

    fn accept_visitor_mut(&mut self, v: &dyn ModuleVisitorMut<Result = ()>) -> Option<()> {
        v.visit_personnel_area(self)
    }
}

pub struct PersonnelAreaSerializerDeserializer;

impl DynDeserializeSeed<dyn Module> for PersonnelAreaSerializerDeserializer {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, str: Intermediate) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let personnel: Vec<Person> =
            serde_intermediate::from_intermediate(&str).map_err(|e| e.to_string())?;
        Ok(Box::new(PersonnelArea { personnel }))
    }
}
