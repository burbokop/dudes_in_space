use crate::module::{Module, ModuleCapability, ModuleId};
use crate::person::PersonId;
use std::cell::RefMut;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// interface through which a module can interact with a vessel it is contained in
pub trait VesselModuleInterface {
    fn add_module(&self, module: Box<dyn Module>);
    fn owner(&self) -> PersonId;
    fn console(&self) -> &dyn VesselConsole;
}

/// interface through which a person can interact with a vessel
pub trait VesselConsole {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>>;
    fn move_to_module(&self, person: PersonId, id: ModuleId) -> Result<(), MoveToModuleError>;
    fn capabilities(&self) -> BTreeSet<ModuleCapability>;
    fn primary_capabilities(&self) -> BTreeSet<ModuleCapability>;
}

#[derive(Debug)]
pub enum MoveToModuleError {
    NotEnoughSpace,
}

impl Display for MoveToModuleError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MoveToModuleError {}