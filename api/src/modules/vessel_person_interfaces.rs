use crate::{Person, PersonId};
use crate::modules::{AssemblyRecipe, AssemblyRecipeSeed, Module, ModuleCapability, ModuleId};
use std::cell::RefMut;

pub trait VesselModuleInterface {
    fn add_module(&self, module:  Box<dyn Module>);
    fn vessel_person_interface(&self) -> &dyn VesselPersonInterface;
}

pub trait VesselPersonInterface {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>>;
    fn move_to_module(&self, person: PersonId, id: ModuleId);
}

pub trait ModulePersonInterface {
    fn id(&self) -> ModuleId;
    // returns index in array. TODO replace with uuid
    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize>;
    fn recipe_output_capabilities(&self, index: usize) -> &[ModuleCapability];
    // returns index in array. TODO replace with uuid
    fn has_resources_for_recipe(&self, index: usize) -> bool;
    fn active_recipe(&self) -> Option<usize>;
    /// inputs index in array. TODO replace with uuid
    /// deploy - if true will attach the produced module to this vessel, false - will store in a nearest module storage
    fn start_assembly(&mut self, index: usize, deploy: bool) -> bool;
    fn interact(&mut self) -> bool;
    fn assembly_recipes(&self) -> &[AssemblyRecipe];
}

pub struct DefaultModulePersonInterface {
    id: ModuleId,
}

impl DefaultModulePersonInterface {
    pub fn new(id: ModuleId) -> Self {
        Self { id }
    }
}

impl ModulePersonInterface for DefaultModulePersonInterface {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize> {
        todo!()
    }

    fn recipe_output_capabilities(&self, index: usize) -> &[ModuleCapability] {
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
        false
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        todo!()
    }
}
