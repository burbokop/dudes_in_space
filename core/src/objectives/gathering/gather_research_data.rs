use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::PersonId;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GatherResearchData {}

impl Objective for GatherResearchData {
    type Error = GatherResearchDataError;

    fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum GatherResearchDataError {}

impl Display for GatherResearchDataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GatherResearchDataError {}
