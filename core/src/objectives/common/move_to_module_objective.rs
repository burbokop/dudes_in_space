use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleConsole, ModuleId};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "move_to_module_objective_stage")]
pub(crate) enum MoveToModuleObjective {
    Move { dst: ModuleId },
    Done,
}

impl MoveToModuleObjective {
    pub(crate) fn new(dst: ModuleId) -> Self {
        Self::Move { dst }
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
        match self {
            MoveToModuleObjective::Move { dst } => {
                if *dst == this_module.id() {
                    *self = Self::Done;
                    Ok(ObjectiveStatus::Done(()))
                } else {
                    this_vessel.move_person_to_module(
                        environment_context.subordination_table(),
                        *this_person.id,
                        *dst,
                    )?;
                    Ok(ObjectiveStatus::InProgress)
                }
            }
            MoveToModuleObjective::Done => Ok(ObjectiveStatus::Done(())),
        }
    }
}

// #[derive(Debug)]
// pub(crate) struct MoveToModuleObjectiveError(MoveToModuleError);
//
// impl Display for MoveToModuleObjectiveError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }
//
// impl Error for MoveToModuleObjectiveError {}
