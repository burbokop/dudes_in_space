use std::fmt::Debug;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use crate::bl::{InputRecipe, Item};
use crate::bl::modules::{Module, ModuleCapability, ModuleTypeId};

pub(crate) trait ModuleFactory : Debug {
    fn type_id(&self) -> ModuleTypeId;
    fn create(&self, recipe: &InputRecipe) -> Box<dyn Module>;
    fn capabilities(&self) -> &[ModuleCapability];
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AssemblyRecipe {
    input: InputRecipe,
    // output: Rc<dyn ModuleFactory>,
}

impl AssemblyRecipe {
    pub(crate)    fn new(
        input: InputRecipe,
        output: Rc<dyn ModuleFactory>,
    ) -> Self {
        // Self { input, output }
        Self { input }
    }
    
    pub(crate) fn create(&self) -> Box<dyn Module> {
        todo!()
        // self.output.create(&self.input)
    }
    pub(crate) fn output_capabilities(&self) -> &[ModuleCapability] { 
        todo!()
        // self.output.capabilities() 
    }
}