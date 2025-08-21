use crate::objectives::crafting::CraftVesselFromScratchObjective;
use crate::objectives::trading::{BuyVesselObjective, BuyVesselObjectiveSeed};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::utils::request::ReqContext;
use dudes_in_space_api::vessel::VesselInternalConsole;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

static TYPE_ID: &str = "AcquireVesselObjective";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "acquire_vessel_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::personal::acquire_vessel_objective::AcquireVesselObjectiveSeed::<'context>)]
pub(crate) enum AcquireVesselObjective {
    CheckPrerequisites {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    },
    #[deserialize_seed_xxx(seeds = [(objective, self.seed.seed.buy_vessel_objective_seed)])]
    BuyVessel {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        objective: BuyVesselObjective,
    },
    CraftVessel {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        objective: CraftVesselFromScratchObjective,
    },
}

#[derive(Clone)]
pub(crate) struct AcquireVesselObjectiveSeed<'context> {
    buy_vessel_objective_seed: BuyVesselObjectiveSeed<'context>,
}

impl<'context> AcquireVesselObjectiveSeed<'context> {
    pub(crate) fn new(context: &'context ReqContext) -> Self {
        Self {
            buy_vessel_objective_seed: BuyVesselObjectiveSeed::new(context),
        }
    }
}

impl AcquireVesselObjective {
    pub(crate) fn new(
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        logger: &mut PersonLogger,
    ) -> Self {
        logger.info(format!(
            "Switched to {}: {:?} {:?}",
            TYPE_ID, needed_capabilities, needed_primary_capabilities
        ));
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
                Err(_) => todo!(),
            },

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
                        needed_capabilities: needed_capabilities.clone(),
                        needed_primary_capabilities: needed_primary_capabilities.clone(),
                        objective: BuyVesselObjective::new(
                            std::mem::take(needed_capabilities),
                            std::mem::take(needed_primary_capabilities),
                            environment_context.request_storage_mut(),
                            logger,
                        ),
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
