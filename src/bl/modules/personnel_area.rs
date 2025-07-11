use std::error::Error;
use crate::bl::{Person, Recipe};
use crate::bl::modules::{AssemblyRecipe, Module, ModuleCapability, ModuleVisitor, VesselPersonInterface};
use serde_intermediate::Intermediate;
use crate::bl::utils::dyn_serde::{DynDeserializeFactory, DynSerialize};

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

impl DynSerialize for PersonnelArea {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box< dyn Error >> {
        serde_intermediate::to_intermediate(&self.personnel).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl Module for PersonnelArea {
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

impl DynDeserializeFactory<dyn Module> for PersonnelAreaSerializerDeserializer {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, str: Intermediate) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let personnel: Vec<Person> = serde_intermediate::from_intermediate(&str).map_err(|e| e.to_string())?;
        Ok(Box::new(PersonnelArea{ personnel }))
    }
}
