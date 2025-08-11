use std::collections::{BTreeMap};
use std::error::Error;
use std::fmt::{Display, Formatter};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemCount, ItemId};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonId, PersonLogger};
use dudes_in_space_api::recipe::{ItemRecipe};
use dudes_in_space_api::vessel::{MoveToModuleError, VesselConsole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "craft_items_objective_stage")]
pub(crate) enum CraftItemsObjective {
    SearchingForCraftingModule {
        needed_items: Vec<(ItemId, ItemCount)>,
    },
    MovingToCraftingModule {
        dst: ModuleId,
        needed_items: Vec<(ItemId, ItemCount)>,
    },
    Crafting {
        needed_items: BTreeMap<ItemId, ItemCount>,
        process_token: Option<ProcessToken>,
    },
    Done,
}

impl CraftItemsObjective {
    pub(crate) fn new(needed_items: Vec<(ItemId, ItemCount)>, logger: &mut PersonLogger) -> Self {
        logger.info(format!(
            "Switched to craft items objective (items: {:?})",
            needed_items,
        ));
        Self::SearchingForCraftingModule { needed_items }
    }

    fn is_recipe_set_suitable(recipes: &[ItemRecipe], mut needed_items: Vec<(ItemId, ItemCount)>) -> bool {
        for r in recipes {
            for (item_id, _) in r.output.clone() {
                if let Some(i) = needed_items.iter().position(|(x, _)| *x == item_id) {
                    needed_items.remove(i);
                }
            }
        }
        needed_items.is_empty()
    }
}

impl Objective for CraftItemsObjective {
    type Error = CraftItemsObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::SearchingForCraftingModule { needed_items } => {
                if let Some(console) = this_module.crafting_console() {
                    if Self::is_recipe_set_suitable(console.item_recipes(), needed_items.clone()) {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            dst: this_module.id(),
                            needed_items: std::mem::take(needed_items),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::ItemCrafting)
                {
                    if Self::is_recipe_set_suitable(
                        crafting_module.item_recipes(),
                        needed_items.clone(),
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");
                        *self = Self::MovingToCraftingModule {
                            dst: crafting_module.id(),
                            needed_items: std::mem::take(needed_items),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftItemsObjectiveError::CanNotFindCraftingModule)
            }
            Self::MovingToCraftingModule {
                dst,
needed_items,
            } => {
                if *dst == this_module.id() {
                    logger.info("Crafting modules...");
                    *self = Self::Crafting {
                        needed_items: BTreeMap::from_iter(std::mem::take(
                            needed_items,
                        )),
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
                                needed_items: std::mem::take(needed_items),
                            };
                            return Ok(ObjectiveStatus::InProgress);
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting {
                needed_items,
                process_token,
            } => match process_token {
                None => {
                    if let Some((item, _)) = needed_items.first_key_value() {
                        let crafting_console = this_module.crafting_console_mut().unwrap();
                        let recipe = crafting_console.recipe_by_output_item(item.clone()).unwrap();
                        assert!(crafting_console.has_resources_for_recipe(recipe));
                        assert!(process_token.is_none());
                        *process_token = Some(crafting_console.start(recipe, false).unwrap());

                        logger.info("Picking recipe for:");
                        for (item, count) in crafting_console.recipe_item_output(recipe).unwrap() {
                            if let Some(needed_count) = needed_items.get_mut(&item) {
                                logger.info(format!("    {:?}", item));
                                if *needed_count > count {
                                    *needed_count -= count;
                                } else {
                                    needed_items.remove(&item);
                                }
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
                        return if needed_items.is_empty()
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
pub(crate) enum CraftItemsObjectiveError {
    CanNotFindCraftingModule,
}

impl Display for CraftItemsObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftItemsObjectiveError {}
