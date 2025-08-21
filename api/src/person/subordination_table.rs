use crate::person::PersonId;
use crate::vessel::VesselConsole;

pub struct SubordinationTable {}

pub enum VesselPermissions {
    Enter,
    Operate,
    Pilot,
}

impl SubordinationTable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn has_permission(
        &self,
        person_id: PersonId,
        vessel: &dyn VesselConsole,
        permission: VesselPermissions,
    ) -> bool {
        todo!()
    }
}
