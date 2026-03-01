use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleConsole, ModuleId};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "move_to_module_objective_stage")]
pub(crate) struct MoveToModuleObjective {
    dst: ModuleId,
}

impl MoveToModuleObjective {
    pub(crate) fn new(dst: ModuleId) -> Self {
        Self { dst }
    }
}

impl Objective for MoveToModuleObjective {
    type Result = ();
    type Error = MoveToModuleError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        if self.dst == this_module.id() {
            Ok(ObjectiveStatus::Done(()))
        } else {
            this_vessel.move_person_to_module(
                environment_context.subordination_table(),
                *this_person.id,
                self.dst,
            )?;
            Ok(ObjectiveStatus::InProgress)
        }
    }
}
