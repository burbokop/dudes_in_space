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
#[serde(tag = "state")]
enum State {
    SearchingForCraftingModule,
    MovingToCraftingModule {
        dst: ModuleId,
    },
    Crafting {
        process_token: Option<ProcessToken>,
        interrupted: bool,
    },
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CraftItemsByHashObjective {
    args: CraftItemsByHashObjectiveArgs,
    state: State,
}

impl CraftItemsByHashObjective {
    pub(crate) fn new(args: CraftItemsByHashObjectiveArgs, logger: &mut PersonLogger) -> Self {
        logger.info(format!(
            "Switched to craft items by hash objective (args: {:?})",
            args,
        ));
        Self {
            args,
            state: State::SearchingForCraftingModule,
        }
    }

    fn is_recipe_set_suitable(recipes: &[ItemRecipe], hash: ItemRecipeHash) -> bool {
        recipes.iter().find(|r| r.default_hash() == hash).is_some()
    }

    pub(crate) fn args(&self) -> &CraftItemsByHashObjectiveArgs {
        &self.args
    }

    pub(crate) fn is_interrupted(&self) -> bool {
        match &self.state {
            State::Crafting { interrupted, .. } => *interrupted,
            _ => false,
        }
    }

    pub(crate) fn resume(&mut self) {
        match &mut self.state {
            State::Crafting { interrupted, .. } => *interrupted = false,
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
        match &mut self.state {
            State::SearchingForCraftingModule => {
                if let Some(console) = this_module.crafting_console() {
                    if Self::is_recipe_set_suitable(console.item_recipes(), self.args.recipe_hash) {
                        logger.info("Moving to crafting module...");
                        self.state = State::MovingToCraftingModule {
                            dst: this_module.id(),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::ItemCrafting)
                {
                    if Self::is_recipe_set_suitable(
                        crafting_module.item_recipes(),
                        self.args.recipe_hash,
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");
                        self.state = State::MovingToCraftingModule {
                            dst: crafting_module.id(),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftItemsByHashObjectiveError::CanNotFindCraftingModule)
            }
            State::MovingToCraftingModule { dst } => {
                if *dst == this_module.id() {
                    logger.info("Crafting modules...");
                    self.state = State::Crafting {
                        interrupted: self.args.start_interrupted,
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
                        Err(MoveToModuleError::ModuleNotFound) => {
                            Err(Self::Error::CanNotFindCraftingModule)
                        }
                        Err(MoveToModuleError::PermissionDenied) => todo!(),
                        Err(MoveToModuleError::NotEnoughSpace) => {
                            logger.info(
                                "Not enough space in crafting module. Searching another one...",
                            );
                            self.state = State::SearchingForCraftingModule;
                            Ok(ObjectiveStatus::InProgress)
                        }
                    }
                }
            }
            State::Crafting {
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

                    if limit_reached(&all_storages_content, self.args.output_limit.clone()) {
                        return Ok(if self.args.done_if_reached_limit {
                            ObjectiveStatus::Done(self.state = State::Done)
                        } else {
                            ObjectiveStatus::InProgress
                        });
                    }

                    let crafting_console = this_module.crafting_console_mut().unwrap();
                    let recipe_index = crafting_console
                        .recipe_by_hash(self.args.recipe_hash)
                        .unwrap();

                    if !crafting_console.has_resources_for_recipe(recipe_index) {
                        return if self.args.err_if_lack_ingredients {
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
                        if self.args.interrupt_after_each_craft {
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
            State::Done => Ok(ObjectiveStatus::Done(())),
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
        match &self.state {
            State::SearchingForCraftingModule { .. } => write!(f, "SearchingForCraftingModule"),
            State::MovingToCraftingModule { .. } => write!(f, "MovingToCraftingModule"),
            State::Crafting { interrupted, .. } => {
                if *interrupted {
                    write!(f, "Crafting (interrupted)")
                } else {
                    write!(f, "Crafting")
                }
            }
            State::Done => write!(f, "Done"),
        }
    }
}

fn limit_reached(content: &ItemStorageContent, items_limit: BTreeMap<ItemId, ItemCount>) -> bool {
    items_limit
        .into_iter()
        .any(|(item, limit)| content.count(item) >= limit)
}
