use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::recipe::AssemblyRecipe;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use dudes_in_space_api::finance::Money;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CraftModulesObjectiveOptions {
    pub deploy: bool,
    pub wait_if_has_no_ingredients: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CraftingProcess {
    token: ProcessToken,
    cost_price: Money,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "craft_modules_objective_stage")]
pub(crate) enum CraftModulesObjective {
    SearchingForCraftingModule {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        options: CraftModulesObjectiveOptions,
    },
    MovingToCraftingModule {
        dst: ModuleId,
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        options: CraftModulesObjectiveOptions,
    },
    Crafting {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        options: CraftModulesObjectiveOptions,
        process: Option<CraftingProcess>,
    },
    Done,
}

impl CraftModulesObjective {
    pub(crate) fn new(
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        options: CraftModulesObjectiveOptions,
        logger: &mut PersonLogger,
    ) -> Self {
        logger.info(format!(
            "Switched to craft modules objective (caps: {:?}, primary caps: {:?})",
            needed_capabilities, needed_primary_capabilities
        ));
        Self::SearchingForCraftingModule {
            needed_capabilities: needed_capabilities.into_iter().collect(),
            needed_primary_capabilities: needed_primary_capabilities.into_iter().collect(),
            options,
        }
    }

    fn is_recipe_set_suitable(
        recipes: &[AssemblyRecipe],
        mut needed_capabilities: BTreeSet<ModuleCapability>,
        mut needed_primary_capabilities: BTreeSet<ModuleCapability>,
    ) -> bool {
        (|| {
            for r in recipes {
                for cap in r.output_description().capabilities() {
                    needed_capabilities.remove(cap);
                }
            }
            needed_capabilities.is_empty()
        })() && (|| {
            for r in recipes {
                for cap in r.output_description().primary_capabilities() {
                    needed_primary_capabilities.remove(cap);
                }
            }
            needed_primary_capabilities.is_empty()
        })()
    }
}

pub struct CraftModulesObjectiveResult {
    pub cost_price: Money
}

impl Objective for CraftModulesObjective {
    type Result = ();
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
            Self::SearchingForCraftingModule {
                needed_capabilities,
                needed_primary_capabilities,
                options,
            } => {
                if let Some(assembly_console) = this_module.crafting_console() {
                    logger.info(format!(
                        "Checking if module (id: {}, type: {}) is suitable for crafting modules...",
                        this_module.id(),
                        this_module.type_id()
                    ));
                    if Self::is_recipe_set_suitable(
                        assembly_console.assembly_recipes(),
                        needed_capabilities.clone(),
                        needed_primary_capabilities.clone(),
                    ) {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            dst: this_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                             options: std::mem::take( options),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::ModuleCrafting)
                {
                    logger.info(format!(
                        "Checking if module (id: {}, type: {}) is suitable for crafting modules...",
                        crafting_module.id(),
                        crafting_module.type_id()
                    ));
                    if Self::is_recipe_set_suitable(
                        crafting_module.assembly_recipes(),
                        needed_capabilities.clone(),
                        needed_primary_capabilities.clone(),
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            dst: crafting_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                            options: std::mem::take( options),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftModulesObjectiveError::CanNotFindCraftingModule)
            }
            Self::MovingToCraftingModule {
                dst,
                needed_capabilities,
                needed_primary_capabilities,
                options,
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
                        options: std::mem::take(options),
                        process_token: None,
                    };
                    Ok(ObjectiveStatus::InProgress)
                } else {
                    logger.info("Entering crafting module...");
                    match this_vessel.move_person_to_module(
                        environment_context.subordination_table(),
                        *this_person.id,
                        *dst,
                    ) {
                        Ok(_) => Ok(ObjectiveStatus::InProgress),
                        Err(MoveToModuleError::ModuleNotFound) => todo!(),
                        Err(MoveToModuleError::PermissionDenied) => {
                            Err(Self::Error::PermissionDenied)
                        }
                        Err(MoveToModuleError::NotEnoughSpace) => {
                            logger.info(
                                "Not enough space in crafting module. Searching another one...",
                            );
                            *self = Self::SearchingForCraftingModule {
                                needed_capabilities: std::mem::take(needed_capabilities),
                                needed_primary_capabilities: std::mem::take(
                                    needed_primary_capabilities,
                                ),
                                options: std::mem::take(options),
                            };
                            Ok(ObjectiveStatus::InProgress)
                        }
                    }
                }
            }
            Self::Crafting {
                needed_capabilities,
                needed_primary_capabilities,
                options,
                process,
            } => match process {
                None => {
                    if let Some(cap) = needed_capabilities.first() {
                        let assembly_console = this_module.crafting_console_mut().unwrap();
                        let recipe = assembly_console.recipe_by_output_capability(*cap).unwrap();
                        assert!(assembly_console.has_resources_for_recipe(recipe));
                        assert!(process_token.is_none());
                        *process_token = Some(assembly_console.start(recipe, options.deploy).unwrap());

                        logger.info("Picking recipe for:");
                        for c in assembly_console
                            .recipe_output_description(recipe)
                            .capabilities()
                        {
                            if needed_capabilities.remove(c) {
                                logger.info(format!("    {:?}", c));
                            }
                        }
                        for c in assembly_console
                            .recipe_output_description(recipe)
                            .primary_capabilities()
                        {
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
                        *process_token = Some(assembly_console.start(recipe, options.deploy).unwrap());
                        logger.info("Picking recipe for:");
                        for c in assembly_console
                            .recipe_output_description(recipe)
                            .capabilities()
                        {
                            if needed_capabilities.remove(c) {
                                logger.info(format!("    {:?}", c));
                            }
                        }
                        for c in assembly_console
                            .recipe_output_description(recipe)
                            .primary_capabilities()
                        {
                            if needed_primary_capabilities.remove(c) {
                                logger.info(format!("    {:?} (primary)", c));
                            }
                        }
                        return Ok(ObjectiveStatus::InProgress);
                    }

                    logger.info("Done crafting modules.");
                    *self = Self::Done;
                    Ok(ObjectiveStatus::Done(()))
                }
                Some(CraftingProcess { token, cost_price }) => {
                    
                    todo!("Do something with cost_price");
                    
                    if token
                        .is_completed(environment_context.process_token_context())
                        .unwrap_or(true)
                    {
                        return if needed_capabilities.is_empty()
                            && needed_primary_capabilities.is_empty()
                        {
                            logger.info("Done crafting modules.");
                            *self = Self::Done;
                            Ok(ObjectiveStatus::Done(()))
                        } else {
                            *process = None;
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
            Self::Done => Ok(ObjectiveStatus::Done(())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum CraftModulesObjectiveError {
    CanNotFindCraftingModule,
    PermissionDenied,
}

impl Display for CraftModulesObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftModulesObjectiveError {}
