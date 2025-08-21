use crate::objectives::crafting::CraftVesselFromScratchObjective;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum AcquireVesselObjective {
    CheckPrerequisites {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    },
    BuyVessel {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    },
    CraftVessel {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        objective: CraftVesselFromScratchObjective,
    },
}

impl AcquireVesselObjective {
    pub(crate) fn new(
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    ) -> Self {
        Self::CheckPrerequisites {
            needed_capabilities,
            needed_primary_capabilities,
        }
    }
}

impl Objective for AcquireVesselObjective {
    type Error = AcquireVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CheckPrerequisites {
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                *self = Self::CraftVessel {
                    needed_capabilities: needed_capabilities.clone(),
                    needed_primary_capabilities: needed_primary_capabilities.clone(),
                    objective: CraftVesselFromScratchObjective::new(
                        std::mem::take(needed_capabilities),
                        std::mem::take(needed_primary_capabilities),
                    ),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::BuyVessel {
                needed_capabilities,
                needed_primary_capabilities,
            } => todo!(),
            Self::CraftVessel {
                needed_capabilities,
                needed_primary_capabilities,
                objective,
            } => match objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done) => Ok(ObjectiveStatus::Done),
                Err(err) => {
                    *self = Self::BuyVessel {
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
            },
        }
    }
}

#[derive(Debug)]
pub(crate) enum AcquireVesselObjectiveError {}

impl Display for AcquireVesselObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for AcquireVesselObjectiveError {}
