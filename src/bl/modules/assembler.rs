use serde::{Deserialize, Serialize};
use crate::bl::modules::{AssemblyRecipe, Module, ModuleCapability, ModuleSerializerDeserializer, ModuleVisitor, PersonnelArea, VesselPersonInterface, WorkerControlPanel};
use crate::bl::{Person, Recipe};

static TYPE_ID: &str = "Assembler";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Assembler {
    operator: Option<Person>,
    recipes: Vec<AssemblyRecipe>,
}

impl WorkerControlPanel for Assembler {
}

impl Assembler {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self { operator: None, recipes: vec![] })
    }
}

impl Module for Assembler {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        if let Some(operator) = &mut self.operator {
            operator.proceed(v)
        }
    }

    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result=()>) -> Option<()> {
        v.visit_assembler(self)
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        &[ModuleCapability::Crafting]
    }

    fn recipes(&self) -> Vec<Recipe> {
        vec![]
    }

    fn assembly_recipes(&self) -> Vec<AssemblyRecipe> {
        todo!()
    }
}

pub(crate) struct AssemblerDeserializer;

impl ModuleSerializerDeserializer for AssemblerDeserializer {
    fn type_id(&self) -> String {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, str: String) -> Result<Box<dyn Module>, String> {
        let assembler: Assembler = serde_json::from_str(str.as_str()).map_err(|e| e.to_string())?;
        Ok(Box::new(assembler))
    }
}
