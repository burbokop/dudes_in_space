use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonId, PersonLogger};
use dudes_in_space_api::recipe::AssemblyRecipe;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselConsole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "craft_modules_objective_stage")]
pub(crate) enum CraftModulesObjective {
    SearchingForCraftingModule {
        this_person: PersonId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        deploy: bool,
    },
    MovingToCraftingModule {
        this_person: PersonId,
        dst: ModuleId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        deploy: bool,
    },
    Crafting {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        deploy: bool,
        process_token: Option<ProcessToken>,
    },
    Done,
}

impl CraftModulesObjective {
    pub(crate) fn new(
        this_person: PersonId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        deploy: bool,
        logger: &mut PersonLogger,
    ) -> Self {
        logger.info(format!(
            "Switched to craft modules objective (caps: {:?}, primary caps: {:?})",
            needed_capabilities, needed_primary_capabilities
        ));
        Self::SearchingForCraftingModule {
            this_person,
            needed_capabilities,
            needed_primary_capabilities,
            deploy,
        }
    }

    fn is_recipe_set_suitable(
        recipes: &[AssemblyRecipe],
        mut needed_capabilities: Vec<ModuleCapability>,
        mut needed_primary_capabilities: Vec<ModuleCapability>,
    ) -> bool {
        (|| {
            for r in recipes {
                for cap in r.output_capabilities() {
                    if let Some(i) = needed_capabilities.iter().position(|x| *x == *cap) {
                        needed_capabilities.remove(i);
                    }
                }
            }
            needed_capabilities.is_empty()
        })() && (|| {
            for r in recipes {
                for cap in r.output_primary_capabilities() {
                    if let Some(i) = needed_primary_capabilities.iter().position(|x| *x == *cap) {
                        needed_primary_capabilities.remove(i);
                    }
                }
            }
            needed_primary_capabilities.is_empty()
        })()
    }
}

impl Objective for CraftModulesObjective {
    type Error = CraftModulesObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::SearchingForCraftingModule {
                this_person,
                needed_capabilities,
                needed_primary_capabilities,
                deploy,
            } => {
                if let Some(assembly_console) = this_module.crafting_console() {
                    if Self::is_recipe_set_suitable(
                        assembly_console.assembly_recipes(),
                        needed_capabilities.clone(),
                        needed_primary_capabilities.clone(),
                    ) {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            this_person: this_person.clone(),
                            dst: this_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                            deploy: *deploy,
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::Crafting)
                {
                    if Self::is_recipe_set_suitable(
                        crafting_module.assembly_recipes(),
                        needed_capabilities.clone(),
                        needed_primary_capabilities.clone(),
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            this_person: this_person.clone(),
                            dst: crafting_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                            deploy: *deploy,
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftModulesObjectiveError::CanNotFindCraftingModule)
            }
            Self::MovingToCraftingModule {
                this_person,
                dst,
                needed_capabilities,
                needed_primary_capabilities,
                deploy,
            } => {
                if *dst == this_module.id() {
                    logger.info("Crafting modules...");
                    *self = Self::Crafting {
                        needed_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_capabilities,
                        )),
                        needed_primary_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_primary_capabilities,
                        )),
                        deploy: *deploy,
                        process_token: None,
                    };
                } else {
                    logger.info("Entering crafting module...");
                    match this_vessel.move_person_to_module(*this_person, *dst) {
                        Ok(_) => {}
                        Err(MoveToModuleError::NotEnoughSpace) => {
                            logger.info(
                                "Not enough space in crafting module. Searching another one...",
                            );
                            *self = Self::SearchingForCraftingModule {
                                this_person: this_person.clone(),
                                needed_capabilities: std::mem::take(needed_capabilities),
                                needed_primary_capabilities: std::mem::take(
                                    needed_primary_capabilities,
                                ),
                                deploy: *deploy,
                            };
                            return Ok(ObjectiveStatus::InProgress);
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting {
                needed_capabilities,
                needed_primary_capabilities,
                deploy,
                process_token,
            } => match process_token {
                None => {
                    if let Some(cap) = needed_capabilities.first() {
                        let assembly_console = this_module.crafting_console_mut().unwrap();
                        let recipe = assembly_console.recipe_by_output_capability(*cap).unwrap();
                        assert!(assembly_console.has_resources_for_recipe(recipe));
                        assert!(process_token.is_none());
                        *process_token = Some(assembly_console.start(recipe, *deploy).unwrap());

                        logger.info("Picking recipe for:");
                        for c in assembly_console.recipe_output_capabilities(recipe) {
                            if needed_capabilities.remove(c) {
                                logger.info(format!("    {:?}", c));
                            }
                        }
                        for c in assembly_console.recipe_output_primary_capabilities(recipe) {
                            if needed_primary_capabilities.remove(c) {
                                logger.info(format!("    {:?} (primary)", c));
                            }
                        }
                        return Ok(ObjectiveStatus::InProgress);
                    }

                    if let Some(cap) = needed_primary_capabilities.first() {
                        let assembly_console = this_module.crafting_console_mut().unwrap();
                        let recipe = assembly_console
                            .recipe_by_output_primary_capability(*cap)
                            .unwrap();
                        assert!(assembly_console.has_resources_for_recipe(recipe));
                        assert!(process_token.is_none());
                        *process_token = Some(assembly_console.start(recipe, *deploy).unwrap());
                        logger.info("Picking recipe for:");
                        for c in assembly_console.recipe_output_capabilities(recipe) {
                            if needed_capabilities.remove(c) {
                                logger.info(format!("    {:?}", c));
                            }
                        }
                        for c in assembly_console.recipe_output_primary_capabilities(recipe) {
                            if needed_primary_capabilities.remove(c) {
                                logger.info(format!("    {:?} (primary)", c));
                            }
                        }
                        return Ok(ObjectiveStatus::InProgress);
                    }

                    todo!()
                }
                Some(some_process_token) => {
                    if some_process_token
                        .is_completed(environment_context.process_token_context())
                        .unwrap_or(true)
                    {
                        return if needed_capabilities.is_empty()
                            && needed_primary_capabilities.is_empty()
                        {
                            logger.info("Done crafting modules.");
                            *self = Self::Done;
                            Ok(ObjectiveStatus::Done)
                        } else {
                            *process_token = None;
                            Ok(ObjectiveStatus::InProgress)
                        };
                    }

                    assert!(this_module.in_progress());

                    logger.info("Waiting for assembling to complete...");
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
