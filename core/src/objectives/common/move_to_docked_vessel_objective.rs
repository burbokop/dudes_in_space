use crate::objectives::common::move_to_module_objective::MoveToModuleObjective;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person;
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::{VesselId, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "move_to_docked_vessel_objective_stage")]
pub(crate) enum MoveToDockedVesselObjective {
    SearchVessel {
        vessel_id: VesselId,
    },
    MoveToDockingClamp {
        vessel_id: VesselId,
        objective: MoveToModuleObjective,
    },
    EnterVessel {
        vessel_id: VesselId,
    },
    Done,
}

impl MoveToDockedVesselObjective {
    pub(crate) fn new(vessel_id: VesselId) -> Self {
        Self::SearchVessel { vessel_id }
    }
}

impl Objective for MoveToDockedVesselObjective {
    type Error = MoveToDockedVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::SearchVessel { vessel_id } => {
                if *vessel_id == this_vessel.id() {
                    logger.info("SearchForCockpit");
                    *self = Self::Done;
                    return Ok(ObjectiveStatus::Done);
                }

                if let Some((module_id, connection_id)) = person::utils::find_map_docking_clamp(
                    this_module,
                    this_vessel,
                    |module_id, clamp| {
                        if let Some(connection) = clamp.connection() {
                            if connection.vessel.id() == *vessel_id {
                                return Some((module_id, connection.connector_id));
                            }
                        }
                        None
                    },
                ) {
                    logger.info("Moving to docking clamp...");
                    *self = Self::MoveToDockingClamp {
                        vessel_id: *vessel_id,
                        objective: MoveToModuleObjective::new(module_id),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                todo!()
            }
            Self::MoveToDockingClamp {
                vessel_id,
                objective,
            } => match objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done) => {
                    logger.info("Entering vessel...");
                    *self = Self::EnterVessel {
                        vessel_id: *vessel_id,
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(err) => todo!(),
            },
            Self::EnterVessel { vessel_id } => {
                let connection_id = person::utils::find_docking_clamp_with_vessel_with_id_mut(
                    this_module.docking_clamps_mut(),
                    *vessel_id,
                )
                .unwrap()
                .connection()
                .unwrap()
                .connector_id;

                this_vessel
                    .move_person_to_docked_vessel(
                        environment_context.subordination_table(),
                        this_module,
                        *this_person.id,
                        connection_id,
                    )
                    .unwrap();

                *self = Self::Done;
                Ok(ObjectiveStatus::Done)
            }
            Self::Done => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum MoveToDockedVesselObjectiveError {}

impl Display for MoveToDockedVesselObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MoveToDockedVesselObjectiveError {}
