use crate::objectives::crafting::{
    CraftModulesObjective, CraftModulesObjectiveError, CraftModulesObjectiveOptions,
};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::Money;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson, tie};
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

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
    Done {
        result: RequireModulesObjectiveResult,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequireModulesObjectiveResult {
    pub cost_price: Money,
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
    type Result = RequireModulesObjectiveResult;
    type Error = CraftModulesObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::SearchingForRequireModules {
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                let this_caps = tie(this_module, this_vessel).capabilities();
                needed_capabilities.retain(|x| !this_caps.contains(x));

                let this_primary_caps = tie(this_module, this_vessel).primary_capabilities();
                needed_primary_capabilities.retain(|x| !this_primary_caps.contains(x));

                *self = Self::Crafting {
                    crafting_objective: CraftModulesObjective::new(
                        std::mem::take(needed_capabilities),
                        std::mem::take(needed_primary_capabilities),
                        CraftModulesObjectiveOptions {
                            deploy: true,
                            wait_if_has_no_ingredients: false,
                        },
                        logger,
                    ),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting { crafting_objective } => match crafting_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done(result)) => {
                    let result = RequireModulesObjectiveResult {
                        cost_price: result.cost_price,
                    };
                    *self = Self::Done {
                        result: result.clone(),
                    };
                    Ok(ObjectiveStatus::Done(result))
                }
                Err(_) => todo!(),
            },
            Self::Done { result } => todo!("result: {:?}", result),
        }
    }
}

impl Display for RequireModulesObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
