use crate::objectives::common::MoveToModuleObjective;
use dudes_in_space_api::environment::{Cycle, EnvironmentContext};
use dudes_in_space_api::item::{ItemCount, ItemId, ItemStorageContent, StorageRole};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::recipe::{ItemRecipe, ItemRecipeHash, OutputItemRecipe};
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum BehaviourIfLackIngredients {
    WaitIndefinitely,
    WaitFor { cycles: Cycle },
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CraftItemsByHashObjectiveArgs {
    pub(crate) recipe_hash: ItemRecipeHash,
    /// Stop producing when reached the limit
    pub(crate) output_limit: BTreeMap<ItemId, ItemCount>,
    /// Wait indefinitely if false
    pub(crate) done_if_reached_limit: bool,
    pub(crate) behaviour_if_lack_ingredients: BehaviourIfLackIngredients,
    pub(crate) interrupt_after_each_craft: bool,
    pub(crate) start_interrupted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "state")]
enum State {
    SearchingForCraftingModule,
    Crafting {
        move_objective: MoveToModuleObjective,
        process_token: Option<ProcessToken>,
        expected_output: OutputItemRecipe,
        interrupted: bool,
        reached_output_limit: bool,
    },
    Done,
}

fn deserialize_data<'de, D>(data: D) -> Result<Option<ModuleId>, D::Error>
where
    D: Deserializer<'de>,
{
    let x = ModuleId::deserialize(data)?;
    Ok(Some(x))
}

fn is_cycle_zero(c: &Cycle) -> bool {
    *c == 0
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CraftItemsByHashObjective {
    args: CraftItemsByHashObjectiveArgs,
    state: State,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_data"
    )]
    crafting_module: Option<ModuleId>,
    #[serde(default, skip_serializing_if = "is_cycle_zero")]
    cycles_without_ingredients: Cycle,
}

impl CraftItemsByHashObjective {
    pub(crate) fn new(args: CraftItemsByHashObjectiveArgs, logger: &mut PersonLogger) -> Self {
        logger.info("Switched to craft items by hash objective");
        Self {
            args,
            state: State::SearchingForCraftingModule,
            crafting_module: None,
            cycles_without_ingredients: 0,
        }
    }

    fn find_recipe(recipes: &[ItemRecipe], hash: ItemRecipeHash) -> Option<&ItemRecipe> {
        recipes.iter().find(|r| r.hash() == hash)
    }

    pub(crate) fn args(&self) -> &CraftItemsByHashObjectiveArgs {
        &self.args
    }

    pub(crate) fn crafting_module(&self) -> Option<ModuleId> {
        self.crafting_module
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
                    if let Some(recipe) =
                        Self::find_recipe(console.item_recipes(), self.args.recipe_hash)
                    {
                        logger.info("Moving to crafting module...");

                        self.crafting_module = Some(this_module.id());
                        self.state = State::Crafting {
                            move_objective: MoveToModuleObjective::new(this_module.id()),
                            interrupted: self.args.start_interrupted,
                            process_token: None,
                            reached_output_limit: false,
                            expected_output: recipe.output.clone(),
                        };

                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::ItemCrafting)
                {
                    if let Some(recipe) =
                        Self::find_recipe(crafting_module.item_recipes(), self.args.recipe_hash)
                        && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");

                        self.crafting_module = Some(crafting_module.id());
                        self.state = State::Crafting {
                            move_objective: MoveToModuleObjective::new(crafting_module.id()),
                            interrupted: self.args.start_interrupted,
                            process_token: None,
                            reached_output_limit: false,
                            expected_output: recipe.output.clone(),
                        };

                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftItemsByHashObjectiveError::CanNotFindCraftingModule)
            }
            State::Crafting {
                move_objective,
                process_token,
                interrupted,
                expected_output,
                reached_output_limit,
            } => match move_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Passive) => todo!(),
                Ok(ObjectiveStatus::Done(_)) => match process_token {
                    None => {
                        if *interrupted {
                            return Ok(ObjectiveStatus::InProgress);
                        }

                        let all_output_storages_content: ItemStorageContent = this_module
                            .storages_by_role(StorageRole::Output)
                            .iter()
                            .map(|x| x.content())
                            .cloned()
                            .sum();

                        if limit_reached(&all_output_storages_content, &self.args.output_limit) {
                            *reached_output_limit = true;
                            return Ok(if self.args.done_if_reached_limit {
                                self.crafting_module = None;
                                ObjectiveStatus::Done(self.state = State::Done)
                            } else {
                                ObjectiveStatus::Passive
                            });
                        }
                        *reached_output_limit = false;

                        let crafting_console = this_module.crafting_console_mut().unwrap();
                        let recipe_index = crafting_console
                            .recipe_by_hash(self.args.recipe_hash)
                            .unwrap();

                        if crafting_console.has_resources_for_recipe(recipe_index) {
                            self.cycles_without_ingredients = 0;
                        } else {
                            return match self.args.behaviour_if_lack_ingredients {
                                BehaviourIfLackIngredients::WaitIndefinitely => {
                                    Ok(ObjectiveStatus::Passive)
                                }
                                BehaviourIfLackIngredients::WaitFor { cycles } => {
                                    self.cycles_without_ingredients += 1;
                                    if self.cycles_without_ingredients > cycles {
                                        Err(CraftItemsByHashObjectiveError::LackIngredients)
                                    } else {
                                        Ok(ObjectiveStatus::Passive)
                                    }
                                }
                                BehaviourIfLackIngredients::Error => {
                                    Err(CraftItemsByHashObjectiveError::LackIngredients)
                                }
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
                Err(err) => todo!("{:?}", err),
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
        let c = self.cycles_without_ingredients;
        match &self.state {
            State::SearchingForCraftingModule { .. } => write!(f, "SearchingForCraftingModule"),
            State::Crafting {
                interrupted,
                process_token,
                expected_output,
                reached_output_limit,
                ..
            } => {
                if *interrupted {
                    write!(f, "Crafting (interrupted)")
                } else if c > 0 {
                    match self.args.behaviour_if_lack_ingredients {
                        BehaviourIfLackIngredients::WaitIndefinitely => {
                            write!(
                                f,
                                "Crafting {}. lack ingredients. cycles to interrupt: {} / indefinitely",
                                expected_output, c
                            )
                        }
                        BehaviourIfLackIngredients::WaitFor { cycles } => {
                            write!(
                                f,
                                "Crafting {}. lack ingredients. cycles to interrupt: {} / {}",
                                expected_output, c, cycles
                            )
                        }
                        BehaviourIfLackIngredients::Error => unreachable!(),
                    }
                } else if *reached_output_limit {
                    write!(f, "Crafting (Waiting for output storage to be free)")
                } else {
                    match process_token {
                        Some(process_token) => write!(f, "Crafting {{ pt: {:?} }}", process_token),
                        None => write!(f, "Crafting (No token)"),
                    }
                }
            }
            State::Done => write!(f, "Done"),
        }
    }
}

fn limit_reached(content: &ItemStorageContent, items_limit: &BTreeMap<ItemId, ItemCount>) -> bool {
    items_limit
        .into_iter()
        .any(|(item, limit)| content.count(item) >= *limit)
}
