use crate::objectives::crafting::{CraftModulesObjective, CraftModulesObjectiveError};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "require_modules_objective_stage")]
pub(crate) enum RequireModulesObjective {
    SearchingForRequireModules {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    },
    Crafting {
        crafting_objective: CraftModulesObjective,
    },
}

impl RequireModulesObjective {
    pub(crate) fn new(
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        logger: &mut PersonLogger,
    ) -> Self {
        logger.info(format!(
            "Switched to require modules objective (caps: {:?}, primary caps: {:?})",
            needed_capabilities, needed_primary_capabilities
        ));
        Self::SearchingForRequireModules {
            needed_capabilities,
            needed_primary_capabilities,
        }
    }
}

impl Objective for RequireModulesObjective {
    type Error = CraftModulesObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::SearchingForRequireModules {
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                let this_caps = person::utils::this_vessel_capabilities(this_module, this_vessel);
                needed_capabilities.retain(|x| !this_caps.contains(x));

                let this_primary_caps =
                    person::utils::this_vessel_primary_capabilities(this_module, this_vessel);
                needed_primary_capabilities.retain(|x| !this_primary_caps.contains(x));

                *self = Self::Crafting {
                    crafting_objective: CraftModulesObjective::new(
                        std::mem::take(needed_capabilities),
                        std::mem::take(needed_primary_capabilities),
                        true,
                        logger,
                    ),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting { crafting_objective } => crafting_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ),
        }
    }
}
