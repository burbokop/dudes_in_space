use crate::module::{Module, ModuleCapability, ModuleConsole, ModuleId};
use crate::person::{PersonId, SubordinationTable, VesselPermissions};
use crate::vessel::{DockingConnectorId, VesselId};
use std::cell::{Ref, RefMut};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// interface through which a module can interact with a vessel it is contained in
pub trait VesselModuleInterface {
    fn add_module(&self, module: Box<dyn Module>);
    fn console(&self) -> &dyn VesselInternalConsole;
}

/// interface through which a person can interact with a vessel
pub trait VesselConsole {
    fn id(&self) -> VesselId;
    fn owner(&self) -> PersonId;
    fn capabilities(&self) -> BTreeSet<ModuleCapability>;
    fn primary_capabilities(&self) -> BTreeSet<ModuleCapability>;
    fn permissions(&self) -> &VesselPermissions;
}

pub trait VesselInternalConsole: VesselConsole {
    fn modules_with_capability<'a>(&'a self, cap: ModuleCapability) -> Vec<Ref<'a, dyn Module>>;
    fn modules_with_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> Vec<RefMut<'a, dyn Module>>;
    fn modules_with_primary_capability<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> Vec<Ref<'a, dyn Module>>;

    fn modules_with_primary_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> Vec<RefMut<'a, dyn Module>>;

    fn move_person_to_module(
        &self,
        subordination_table: &SubordinationTable,
        person_id: PersonId,
        module_id: ModuleId,
    ) -> Result<(), MoveToModuleError>;
    fn move_person_to_docked_vessel(
        &self,
        subordination_table: &SubordinationTable,
        this_module: &dyn ModuleConsole,
        person_id: PersonId,
        connector_id: DockingConnectorId,
    ) -> Result<(), MoveToDockedVesselError>;
}

#[derive(Debug)]
pub enum MoveToModuleError {
    ModuleNotFound,
    NotEnoughSpace,
    PermissionDenied,
}

impl Display for MoveToModuleError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MoveToModuleError {}

#[derive(Debug)]
pub enum MoveToDockedVesselError {
    NotEnoughSpace,
}

impl Display for MoveToDockedVesselError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MoveToDockedVesselError {}
