use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemCount, ItemId, ItemStorageContent, StorageRole};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::recipe::{ItemRecipe, ItemRecipeHash};
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct CraftItemsByHashObjectiveArgs {
    pub(crate) recipe_hash: ItemRecipeHash,
    /// Stop producing when reached the limit
    pub(crate) output_limit: BTreeMap<ItemId, ItemCount>,
    /// Wait indefinitely if false
    pub(crate) done_if_reached_limit: bool,
    /// Wait indefinitely if false
    pub(crate) err_if_lack_ingredients: bool,
    pub(crate) interrupt_after_each_craft: bool,
    pub(crate) start_interrupted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "craft_items_by_hash_objective_stage")]
pub(crate) enum CraftItemsByHashObjective {
    SearchingForCraftingModule {
        args: CraftItemsByHashObjectiveArgs,
    },
    MovingToCraftingModule {
        args: CraftItemsByHashObjectiveArgs,
        dst: ModuleId,
    },
    Crafting {
        args: CraftItemsByHashObjectiveArgs,
        process_token: Option<ProcessToken>,
        interrupted: bool,
    },
    Done,
}

impl CraftItemsByHashObjective {
    pub(crate) fn new(args: CraftItemsByHashObjectiveArgs, logger: &mut PersonLogger) -> Self {
        logger.info(format!(
            "Switched to craft items by hash objective (args: {:?})",
            args,
        ));
        Self::SearchingForCraftingModule { args }
    }

    fn is_recipe_set_suitable(recipes: &[ItemRecipe], hash: ItemRecipeHash) -> bool {
        recipes.iter().find(|r| r.default_hash() == hash).is_some()
    }

    pub(crate) fn is_interrupted(&self) -> bool {
        match self {
            Self::Crafting { interrupted, .. } => *interrupted,
            _ => false,
        }
    }

    pub(crate) fn resume(&mut self) {
        match self {
            Self::Crafting { interrupted, .. } => *interrupted = false,
            _ => {}
        }
    }
}

impl Objective for CraftItemsByHashObjective {
    type Result = ();
    type Error = CraftItemsByHashObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::SearchingForCraftingModule { args } => {
                if let Some(console) = this_module.crafting_console() {
                    if Self::is_recipe_set_suitable(console.item_recipes(), args.recipe_hash) {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            dst: this_module.id(),
                            args: std::mem::take(args),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::ItemCrafting)
                {
                    if Self::is_recipe_set_suitable(
                        crafting_module.item_recipes(),
                        args.recipe_hash,
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            dst: crafting_module.id(),
                            args: std::mem::take(args),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftItemsByHashObjectiveError::CanNotFindCraftingModule)
            }
            Self::MovingToCraftingModule { dst, args } => {
                if *dst == this_module.id() {
                    logger.info("Crafting modules...");
                    *self = Self::Crafting {
                        interrupted: args.start_interrupted,
                        args: std::mem::take(args),
                        process_token: None,
                    };
                } else {
                    logger.info("Entering crafting module...");
                    match this_vessel.move_person_to_module(
                        environment_context.subordination_table(),
                        *this_person.id,
                        *dst,
                    ) {
                        Ok(_) => {}
                        Err(MoveToModuleError::ModuleNotFound) => todo!(),
                        Err(MoveToModuleError::PermissionDenied) => todo!(),
                        Err(MoveToModuleError::NotEnoughSpace) => {
                            logger.info(
                                "Not enough space in crafting module. Searching another one...",
                            );
                            *self = Self::SearchingForCraftingModule {
                                args: std::mem::take(args),
                            };
                            return Ok(ObjectiveStatus::InProgress);
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting {
                args,
                process_token,
                interrupted,
            } => match process_token {
                None => {
                    if *interrupted {
                        return Ok(ObjectiveStatus::InProgress);
                    }

                    let all_storages_content: ItemStorageContent = this_module
                        .storages_by_role(StorageRole::Output)
                        .iter()
                        .map(|x| x.content())
                        .cloned()
                        .sum();

                    if limit_reached(&all_storages_content, args.output_limit.clone()) {
                        return Ok(if args.done_if_reached_limit {
                            ObjectiveStatus::Done(*self = Self::Done)
                        } else {
                            ObjectiveStatus::InProgress
                        });
                    }

                    let crafting_console = this_module.crafting_console_mut().unwrap();
                    let recipe_index = crafting_console.recipe_by_hash(args.recipe_hash).unwrap();

                    if !crafting_console.has_resources_for_recipe(recipe_index) {
                        return if args.err_if_lack_ingredients {
                            Err(CraftItemsByHashObjectiveError::LackIngredients)
                        } else {
                            Ok(ObjectiveStatus::InProgress)
                        };
                    }

                    assert!(process_token.is_none());
                    *process_token = Some(crafting_console.start(recipe_index, false).unwrap());

                    Ok(ObjectiveStatus::InProgress)
                }
                Some(some_process_token) => {
                    if *interrupted {
                        return Ok(ObjectiveStatus::InProgress);
                    }

                    if some_process_token
                        .is_completed(environment_context.process_token_context())
                        .unwrap_or(true)
                    {
                        *process_token = None;
                        if args.interrupt_after_each_craft {
                            *interrupted = true;
                        }
                        return Ok(ObjectiveStatus::InProgress);
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

#[derive(Debug)]
pub(crate) enum CraftItemsByHashObjectiveError {
    CanNotFindCraftingModule,
    LackIngredients,
}

impl Display for CraftItemsByHashObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftItemsByHashObjectiveError {}

impl Display for CraftItemsByHashObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SearchingForCraftingModule { .. } => write!(f, "SearchingForCraftingModule"),
            Self::MovingToCraftingModule { .. } => write!(f, "MovingToCraftingModule"),
            Self::Crafting { interrupted, .. } => {
                if *interrupted {
                    write!(f, "Crafting (interrupted)")
                } else {
                    write!(f, "Crafting")
                }
            }
            Self::Done => write!(f, "Done"),
        }
    }
}

fn limit_reached(content: &ItemStorageContent, items_limit: BTreeMap<ItemId, ItemCount>) -> bool {
    items_limit
        .into_iter()
        .any(|(item, limit)| content.count(item) >= limit)
}
