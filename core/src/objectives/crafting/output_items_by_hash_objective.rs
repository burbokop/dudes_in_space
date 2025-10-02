use crate::objectives::common::MoveToModuleObjective;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemCount, ItemId, ItemStorageContent, StorageRole};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::recipe::{OutputItemRecipe, OutputItemRecipeHash};
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OutputItemsByHashObjectiveArgs {
    pub(crate) recipe_hash: OutputItemRecipeHash,
    /// Stop producing when reached the limit
    pub(crate) output_limit: BTreeMap<ItemId, ItemCount>,
    /// Wait indefinitely if false
    pub(crate) done_if_reached_limit: bool,
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
        interrupted: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OutputItemsByHashObjective {
    args: OutputItemsByHashObjectiveArgs,
    state: State,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_data"
    )]
    crafting_module: Option<ModuleId>,
}

impl OutputItemsByHashObjective {
    pub(crate) fn new(args: OutputItemsByHashObjectiveArgs, logger: &mut PersonLogger) -> Self {
        logger.info(format!(
            "Switched to output items by hash objective (args: {:?})",
            args,
        ));
        Self {
            args,
            state: State::SearchingForCraftingModule,
            crafting_module: None,
        }
    }

    fn is_recipe_set_suitable(recipes: &[OutputItemRecipe], hash: OutputItemRecipeHash) -> bool {
        recipes.iter().find(|r| r.hash() == hash).is_some()
    }

    pub(crate) fn args(&self) -> &OutputItemsByHashObjectiveArgs {
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

impl Objective for OutputItemsByHashObjective {
    type Result = ();
    type Error = OutputItemsByHashObjectiveError;

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
                    if Self::is_recipe_set_suitable(
                        console.output_item_recipes(),
                        self.args.recipe_hash,
                    ) {
                        logger.info("Moving to crafting module...");

                        self.crafting_module = Some(this_module.id());
                        self.state = State::Crafting {
                            move_objective: MoveToModuleObjective::new(this_module.id()),
                            interrupted: self.args.start_interrupted,
                            process_token: None,
                        };

                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::ItemProduction)
                {
                    if Self::is_recipe_set_suitable(
                        crafting_module.output_item_recipes(),
                        self.args.recipe_hash,
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");

                        self.crafting_module = Some(crafting_module.id());
                        self.state = State::Crafting {
                            move_objective: MoveToModuleObjective::new(crafting_module.id()),
                            interrupted: self.args.start_interrupted,
                            process_token: None,
                        };

                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(OutputItemsByHashObjectiveError::CanNotFindCraftingModule)
            }
            State::Crafting {
                move_objective,
                process_token,
                interrupted,
            } => match move_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
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
                            return Ok(if self.args.done_if_reached_limit {
                                self.crafting_module = None;
                                ObjectiveStatus::Done(self.state = State::Done)
                            } else {
                                ObjectiveStatus::InProgress
                            });
                        }

                        let crafting_console = this_module.crafting_console_mut().unwrap();
                        let recipe_index = crafting_console
                            .recipe_by_output_hash(self.args.recipe_hash)
                            .unwrap();

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
pub(crate) enum OutputItemsByHashObjectiveError {
    CanNotFindCraftingModule,
}

impl Display for OutputItemsByHashObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for OutputItemsByHashObjectiveError {}

impl Display for OutputItemsByHashObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.state {
            State::SearchingForCraftingModule { .. } => write!(f, "SearchingForCraftingModule"),
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

fn limit_reached(content: &ItemStorageContent, items_limit: &BTreeMap<ItemId, ItemCount>) -> bool {
    items_limit
        .into_iter()
        .any(|(item, limit)| content.count(item) >= *limit)
}
