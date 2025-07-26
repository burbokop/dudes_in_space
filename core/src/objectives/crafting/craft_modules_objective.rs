use crate::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken, ProcessTokenContext};
use crate::person::PersonId;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::recipe::AssemblyRecipe;
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "crafting_modules_objective_stage")]
pub(crate) enum CraftModulesObjective {
    SearchingForCraftingModule {
        needed_capabilities: Vec<ModuleCapability>,
        deploy: bool,
    },
    MovingToCraftingModule {
        dst: ModuleId,
        needed_capabilities: Vec<ModuleCapability>,
        deploy: bool,
    },
    Crafting {
        needed_capabilities: BTreeSet<ModuleCapability>,
        deploy: bool,
        process_token: Option<ProcessToken>,
    },
    Done,
}
impl CraftModulesObjective {
    pub(crate) fn new(needed_capabilities: Vec<ModuleCapability>, deploy: bool) -> Self {
        Self::SearchingForCraftingModule {
            needed_capabilities,
            deploy,
        }
    }

    fn is_recipe_set_suitable(
        recipes: &[AssemblyRecipe],
        mut needed_caps: Vec<ModuleCapability>,
    ) -> bool {
        for r in recipes {
            for cap in r.output_capabilities() {
                if let Some(i) = needed_caps.iter().position(|x| *x == *cap) {
                    needed_caps.remove(i);
                }
            }
        }
        needed_caps.is_empty()
    }
}

impl Objective for CraftModulesObjective {
    type Error = CraftModulesObjectiveError;

    fn pursue(
        &mut self,
        this_person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::SearchingForCraftingModule {
                needed_capabilities,
                deploy,
            } => {
                if let Some(assembly_console) = this_module.assembly_console() {
                    if Self::is_recipe_set_suitable(
                        assembly_console.recipes(),
                        needed_capabilities.clone(),
                    ) {
                        *self = Self::MovingToCraftingModule {
                            dst: this_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            deploy: *deploy,
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in this_vessel.modules_with_cap(ModuleCapability::Crafting) {
                    if Self::is_recipe_set_suitable(
                        crafting_module.assembly_recipes(),
                        needed_capabilities.clone(),
                    ) {
                        *self = Self::MovingToCraftingModule {
                            dst: crafting_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            deploy: *deploy,
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftModulesObjectiveError::CanNotFindCraftingModule)
            }
            Self::MovingToCraftingModule {
                dst,
                needed_capabilities,
                deploy,
            } => {
                if *dst == this_module.id() {
                    *self = Self::Crafting {
                        needed_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_capabilities,
                        )),
                        deploy: *deploy,
                        process_token: None,
                    };
                } else {
                    this_vessel.move_to_module(this_person, *dst);
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting {
                needed_capabilities,
                deploy,
                process_token,
            } => match process_token {
                None => {
                    if let Some(cap) = needed_capabilities.first() {
                        let assembly_console = this_module.assembly_console_mut().unwrap();
                        if let Some(recipe) = assembly_console.recipe_by_output_capability(*cap) {
                            if assembly_console.has_resources_for_recipe(recipe) {
                                assert!(process_token.is_none());
                                *process_token =
                                    Some(assembly_console.start(recipe, *deploy).unwrap());
                                for c in assembly_console.recipe_output_capabilities(recipe) {
                                    needed_capabilities.remove(c);
                                }
                                Ok(ObjectiveStatus::InProgress)
                            } else {
                                todo!()
                            }
                        } else {
                            todo!()
                        }
                    } else {
                        todo!()
                    }
                }
                Some(some_process_token) => {
                    if some_process_token
                        .is_completed(process_token_context)
                        .unwrap_or(true)
                    {
                        return if needed_capabilities.is_empty() {
                            *self = Self::Done;
                            Ok(ObjectiveStatus::Done)
                        } else {
                            *process_token = None;
                            Ok(ObjectiveStatus::InProgress)
                        };
                    }

                    assert!(this_module.in_progress());
                    if !this_module.interact() {
                        todo!()
                    } else {
                        Ok(ObjectiveStatus::InProgress)
                    }
                }
            },
            Self::Done => Ok(ObjectiveStatus::Done),
        }
    }
}

#[derive(Debug)]
pub(crate) enum CraftModulesObjectiveError {
    CanNotFindCraftingModule,
}

impl Display for CraftModulesObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftModulesObjectiveError {}
