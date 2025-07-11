use crate::bl::{Person, Recipe};
use crate::bl::modules::{AssemblyRecipe, Module, ModuleCapability, ModuleSerializerDeserializer, ModuleVisitor, VesselPersonInterface};
use std::fmt::{Debug, Formatter};

static TYPE_ID: &str = "PersonnelArea";

#[derive(Debug)]
pub(crate) struct PersonnelArea {
    personnel: Vec<Person>,
}

impl PersonnelArea {
    pub(crate) fn new(personnel: Vec<Person>) -> Box<Self> {
        Box::new(Self { personnel })
    }
}

impl Module for PersonnelArea {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> String {
        serde_json::to_string(&self.personnel).unwrap()
    }

    fn proceed(&mut self, v: & dyn VesselPersonInterface) {
        for person in &mut self.personnel {
            person.proceed(v)
        }
    }

    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result=()>) -> Option<()> {
        v.visit_personnel_area(self)
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        &[]
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> Vec<AssemblyRecipe> {
        todo!()
    }
}

pub(crate) struct PersonnelAreaSerializerDeserializer ;

impl ModuleSerializerDeserializer for PersonnelAreaSerializerDeserializer {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, str: String) -> Result<Box<dyn Module>, String> {
        let personnel: Vec<Person> = serde_json::from_str(&str).map_err(|e| e.to_string())?;
        Ok(Box::new(PersonnelArea{ personnel }))
    }
}
