use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{Bank, Money};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person::{
    Objective, ObjectiveStatus, PersonLogger, PurchasedItemsMaxPrices, ThisPerson,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe};
use dudes_in_space_api::utils::math::Zero;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CraftModulesObjectiveOptions {
    pub deploy: bool,
    pub wait_if_has_no_ingredients: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CraftingProcess {
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
        total_cost_price: Money,
    },
    Done {
        result: CraftModulesObjectiveResult,
    },
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CraftModulesObjectiveResult {
    pub cost_price: Money,
}

impl Objective for CraftModulesObjective {
    type Result = CraftModulesObjectiveResult;
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
                            options: std::mem::take(options),
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
                            options: std::mem::take(options),
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
                    let this_person_wallet_id = this_person.finance.wallet().id().clone();
                    *self = Self::Crafting {
                        needed_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_capabilities,
                        )),
                        needed_primary_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_primary_capabilities,
                        )),
                        options: std::mem::take(options),
                        process: None,
                        total_cost_price: Money {
                            currency: this_person.finance.preferred_currency_or_create(
                                environment_context.bank_registry(),
                                Bank::new(
                                    this_person.id.clone(),
                                    this_person_wallet_id,
                                    environment_context.currency_generator().generate_name(
                                        &mut rng(),
                                        environment_context.bank_registry(),
                                        this_person,
                                    ),
                                ),
                            ),
                            amount: Zero::zero(),
                        },
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
                total_cost_price,
            } => match process {
                None => {
                    if let Some(cap) = needed_capabilities.first() {
                        let assembly_console = this_module.crafting_console_mut().unwrap();
                        let recipe_index =
                            assembly_console.recipe_by_output_capability(*cap).unwrap();
                        assert!(assembly_console.has_resources_for_recipe(recipe_index));
                        assert!(process.is_none());

                        *process = Some(CraftingProcess {
                            token: assembly_console
                                .start(recipe_index, options.deploy)
                                .unwrap(),
                            cost_price: calculate_cost_price(
                                assembly_console.recipe_input(recipe_index).unwrap(),
                                this_person.notes.purchased_items_max_prices(),
                            ),
                        });

                        logger.info("Picking recipe for:");
                        for c in assembly_console
                            .recipe_output_description(recipe_index)
                            .capabilities()
                        {
                            if needed_capabilities.remove(c) {
                                logger.info(format!("    {:?}", c));
                            }
                        }
                        for c in assembly_console
                            .recipe_output_description(recipe_index)
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
                        let recipe_index = assembly_console
                            .recipe_by_output_primary_capability(*cap)
                            .unwrap();
                        assert!(assembly_console.has_resources_for_recipe(recipe_index));
                        assert!(process.is_none());

                        *process = Some(CraftingProcess {
                            token: assembly_console
                                .start(recipe_index, options.deploy)
                                .unwrap(),
                            cost_price: calculate_cost_price(
                                assembly_console.recipe_input(recipe_index).unwrap(),
                                this_person.notes.purchased_items_max_prices(),
                            ),
                        });

                        logger.info("Picking recipe for:");
                        for c in assembly_console
                            .recipe_output_description(recipe_index)
                            .capabilities()
                        {
                            if needed_capabilities.remove(c) {
                                logger.info(format!("    {:?}", c));
                            }
                        }
                        for c in assembly_console
                            .recipe_output_description(recipe_index)
                            .primary_capabilities()
                        {
                            if needed_primary_capabilities.remove(c) {
                                logger.info(format!("    {:?} (primary)", c));
                            }
                        }
                        return Ok(ObjectiveStatus::InProgress);
                    }

                    logger.info("Done crafting modules.");
                    let total_cost_price = total_cost_price.clone();
                    *self = Self::Done {
                        result: Self::Result {
                            cost_price: total_cost_price.clone(),
                        },
                    };
                    Ok(ObjectiveStatus::Done(Self::Result {
                        cost_price: total_cost_price,
                    }))
                }
                Some(CraftingProcess { token, cost_price }) => {
                    if token
                        .is_completed(environment_context.process_token_context())
                        .unwrap_or(true)
                    {
                        total_cost_price
                            .add_assign_same_currency(cost_price.clone())
                            .unwrap();
                        return if needed_capabilities.is_empty()
                            && needed_primary_capabilities.is_empty()
                        {
                            logger.info("Done crafting modules.");
                            let total_cost_price = total_cost_price.clone();
                            *self = Self::Done {
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
            Self::Done { result } => Ok(ObjectiveStatus::Done(result.clone())),
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

fn calculate_cost_price(input: InputItemRecipe, prices: &PurchasedItemsMaxPrices) -> Money {
    // maybe should be `Money::sum_as`
    Money::sum_same_currency(input.into_iter().map(|stack| {
        if let Some(price) = prices.stabilized().get(&stack.id) {
            price.clone() * stack.count
        } else {
            todo!()
        }
    }))
    .unwrap()
}
