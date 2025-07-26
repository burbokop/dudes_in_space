use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::PersonId;
use crate::vessel::VesselConsole;
use std::error::Error;
use std::fmt::{Debug, Display};
use dyn_serde::DynSerialize;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ObjectiveStatus {
    InProgress,
    Done,
}

pub(crate) trait Objective {
    type Error;
    fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error>;
}

pub(crate) trait DynObjective: Debug + DynSerialize {
    fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Box<dyn Error>>;
}

impl<T: Objective + Debug + DynSerialize> DynObjective for T {
    fn pursue(
        &mut self, 
        person: PersonId, 
        this_module: &mut dyn ModuleConsole, 
        this_vessel: &dyn VesselConsole, 
        process_token_context: &ProcessTokenContext) -> Result<ObjectiveStatus, Box<dyn Error>> {
        self.pursue(person, this_module, this_vessel, process_token_context)
            .map_err(|e| Box::new(e))
    }
}
