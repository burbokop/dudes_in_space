use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleConsole, ModuleId};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "move_to_module_objective_stage")]
pub(crate) enum MoveToModuleObjective {
    Move { module_id: ModuleId },
    Done,
}

impl MoveToModuleObjective {
    pub(crate) fn new(module_id: ModuleId) -> Self {
        Self::Move { module_id }
    }
}

impl Objective for MoveToModuleObjective {
    type Error = MoveToModuleObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum MoveToModuleObjectiveError {}

impl Display for MoveToModuleObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MoveToModuleObjectiveError {}
