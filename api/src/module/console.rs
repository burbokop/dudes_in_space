use crate::item::ItemStorage;
use crate::module::module::ModuleId;
use crate::module::{ModuleCapability, ModuleStorage, ProcessToken};
use crate::person::Role;
use crate::recipe::AssemblyRecipe;
use crate::utils::math::Vector;
use crate::vessel::DockingClamp;
use std::collections::BTreeSet;
use std::ops::Deref;

/// interface through which a person can interact with a module
pub trait ModuleConsole {
    /// common
    fn id(&self) -> ModuleId;
    fn interact(&mut self) -> bool;
    fn in_progress(&self) -> bool;

    fn assembly_console(&self) -> Option<&dyn AssemblyConsole>;
    fn assembly_console_mut(&mut self) -> Option<&mut dyn AssemblyConsole>;

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole>;
    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole>;

    fn storages(&self) -> &[ItemStorage];
    fn storages_mut(&mut self) -> &mut [ItemStorage];

    fn module_storages(&self) -> &[ModuleStorage];
    fn module_storages_mut(&mut self) -> &mut [ModuleStorage];

    fn docking_clamps(&self) -> &[DockingClamp];
    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp];
}

pub struct DefaultModuleConsole {
    id: ModuleId,
}

impl DefaultModuleConsole {
    pub fn new(id: ModuleId) -> Self {
        Self { id }
    }
}

impl ModuleConsole for DefaultModuleConsole {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn interact(&mut self) -> bool {
        false
    }

    fn in_progress(&self) -> bool {
        todo!()
    }

    fn assembly_console(&self) -> Option<&dyn AssemblyConsole> {
        todo!()
    }

    fn assembly_console_mut(&mut self) -> Option<&mut dyn AssemblyConsole> {
        todo!()
    }

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole> {
        todo!()
    }

    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole> {
        todo!()
    }

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

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }
}

pub trait AssemblyConsole {
    // returns index in array. TODO replace with uuid
    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize>;
    fn recipe_output_capabilities(&self, index: usize) -> &[ModuleCapability];
    // returns index in array. TODO replace with uuid
    fn has_resources_for_recipe(&self, index: usize) -> bool;
    fn active_recipe(&self) -> Option<usize>;
    /// inputs index in array. TODO replace with uuid
    /// deploy - if true will attach the produced module to this vessel, false - will store in a nearest module storage
    fn start(&mut self, index: usize, deploy: bool) -> Option<ProcessToken>;
    fn recipes(&self) -> &[AssemblyRecipe];
}

pub trait DockyardConsole {
    fn start(&mut self, modules: BTreeSet<ModuleId>) -> Option<ProcessToken>;
}

pub(crate) trait CaptainControlPanel {
    fn give_command(&self, role: Role) {}
}

pub(crate) trait NavigatorControlPanel {
    fn scan(&self) {}

    fn plan_route(&self) {}
}

pub(crate) trait GunnerControlPanel {
    fn scan(&self) -> Vector<u32> {
        todo!()
    }

    fn fire_at(&self, vessel_id: u32) {
        todo!()
    }
}
