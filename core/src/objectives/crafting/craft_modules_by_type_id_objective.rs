use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::Money;
use dudes_in_space_api::module::{
    ModuleCapability, ModuleConsole, ModuleId, ModuleTypeId, ProcessToken,
};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::recipe::AssemblyRecipe;
use dudes_in_space_api::utils::math::Zero;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CraftModulesByTypeIdObjectiveArgs {
    pub modules: Vec<ModuleTypeId>,
    pub deploy: bool,
    pub wait_if_has_no_ingredients: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CraftModulesByTypeIdObjectiveCraftingProcess {
    token: ProcessToken,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    cost_price: Option<Money>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "state")]
enum State {
    SearchingForCraftingModule,
    MovingToCraftingModule {
        dst: ModuleId,
    },
    Crafting {
        process: Option<CraftModulesByTypeIdObjectiveCraftingProcess>,
        #[serde(with = "dudes_in_space_api::utils::tagged_option")]
        total_cost_price: Option<Money>,
        modules_left: VecDeque<ModuleTypeId>,
    },
    Done {
        result: CraftModulesByTypeIdObjectiveResult,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CraftModulesByTypeIdObjective {
    args: CraftModulesByTypeIdObjectiveArgs,
    state: State,
}

impl CraftModulesByTypeIdObjective {
    pub(crate) fn new(args: CraftModulesByTypeIdObjectiveArgs, logger: &mut PersonLogger) -> Self {
        logger.info(format!(
            "Switched to craft modules by type id objective: {:?}",
            args.modules
        ));
        Self {
            args,
            state: State::SearchingForCraftingModule,
        }
    }

    fn is_recipe_set_suitable(recipes: &[AssemblyRecipe], mut modules: Vec<ModuleTypeId>) -> bool {
        for r in recipes {
            modules.retain(|x| x != &r.output_description().type_id());
        }
        modules.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CraftModulesByTypeIdObjectiveResult {
    // None if failed to calculate
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    pub cost_price: Option<Money>,
}

impl Objective for CraftModulesByTypeIdObjective {
    type Result = CraftModulesByTypeIdObjectiveResult;
    type Error = CraftModulesByTypeIdObjectiveError;

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
                if let Some(assembly_console) = this_module.crafting_console() {
                    logger.info(format!(
                        "Checking if module (id: {}, type: {}) is suitable for crafting modules...",
                        this_module.id(),
                        this_module.type_id()
                    ));
                    if Self::is_recipe_set_suitable(
                        assembly_console.assembly_recipes(),
                        self.args.modules.clone(),
                    ) {
                        logger.info("Moving to crafting module...");
                        self.state = State::MovingToCraftingModule {
                            dst: this_module.id(),
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
                        self.args.modules.clone(),
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to crafting module...");
                        self.state = State::MovingToCraftingModule {
                            dst: crafting_module.id(),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(CraftModulesByTypeIdObjectiveError::CanNotFindCraftingModule)
            }
            State::MovingToCraftingModule { dst } => {
                if *dst == this_module.id() {
                    logger.info(format!("Crafting modules {:?} ...", self.args.modules));
                    let this_person_wallet_id = this_person.finance.wallet().id().clone();
                    self.state = State::Crafting {
                        process: None,
                        total_cost_price: Some(Money {
                            currency: this_person
                                .preferred_currency_or_create_default(environment_context),
                            amount: Zero::zero(),
                        }),
                        modules_left: self.args.modules.clone().into(),
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
                            self.state = State::SearchingForCraftingModule;
                            Ok(ObjectiveStatus::InProgress)
                        }
                    }
                }
            }
            State::Crafting {
                process,
                total_cost_price,
                modules_left,
            } => match process {
                None => {
                    if let Some(module) = modules_left.front() {
                        let assembly_console = this_module.crafting_console_mut().unwrap();
                        let recipe_index = assembly_console
                            .recipe_by_output_module(module.clone())
                            .unwrap();
                        assert!(assembly_console.has_resources_for_recipe(recipe_index));
                        assert!(process.is_none());

                        *process = Some(CraftModulesByTypeIdObjectiveCraftingProcess {
                            token: assembly_console
                                .start(recipe_index, self.args.deploy)
                                .unwrap(),
                            cost_price: this_person
                                .notes
                                .purchased_items_max_prices()
                                .calculate_cost_price(
                                    assembly_console.recipe_item_input(recipe_index).unwrap(),
                                )
                                .ok(),
                        });

                        return Ok(ObjectiveStatus::InProgress);
                    }

                    logger.info("Done crafting modules.");
                    let total_cost_price = total_cost_price.clone();
                    self.state = State::Done {
                        result: Self::Result {
                            cost_price: total_cost_price.clone(),
                        },
                    };
                    Ok(ObjectiveStatus::Done(Self::Result {
                        cost_price: total_cost_price,
                    }))
                }
                Some(CraftModulesByTypeIdObjectiveCraftingProcess { token, cost_price }) => {
                    if token
                        .is_completed(environment_context.process_token_context())
                        .unwrap_or(true)
                    {
                        modules_left.pop_front();

                        if let (Some(cost_price), Some(total_cost_price)) =
                            (cost_price, total_cost_price.as_mut())
                        {
                            total_cost_price
                                .add_assign_same_currency(cost_price.clone())
                                .unwrap();
                        } else {
                            *total_cost_price = None
                        }

                        return if modules_left.is_empty() {
                            logger.info("Done crafting modules.");
                            let total_cost_price = total_cost_price.clone();
                            self.state = State::Done {
                                result: Self::Result {
                                    cost_price: total_cost_price.clone(),
                                },
                            };
                            Ok(ObjectiveStatus::Done(Self::Result {
                                cost_price: total_cost_price,
                            }))
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
            State::Done { result } => Ok(ObjectiveStatus::Done(result.clone())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum CraftModulesByTypeIdObjectiveError {
    CanNotFindCraftingModule,
    PermissionDenied,
}

impl Display for CraftModulesByTypeIdObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftModulesByTypeIdObjectiveError {}

impl Display for CraftModulesByTypeIdObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.state {
            State::SearchingForCraftingModule => write!(f, "SearchingForCraftingModule"),
            State::MovingToCraftingModule { .. } => write!(f, "MovingToCraftingModule"),
            State::Crafting { .. } => write!(f, "Crafting"),
            State::Done { .. } => write!(f, "Done"),
        }
    }
}
