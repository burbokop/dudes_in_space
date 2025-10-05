use crate::objectives::common::MoveToModuleObjective;
use crate::objectives::crafting::{
    BehaviourIfLackIngredients, CraftItemsByHashObjective, CraftModulesByTypeIdObjective,
    CraftModulesByTypeIdObjectiveArgs, CraftModulesByTypeIdObjectiveError,
    CraftModulesObjectiveError, OutputItemsByHashObjective, OutputItemsByHashObjectiveArgs,
    OutputItemsByHashObjectiveError,
};
use crate::objectives::crafting::{
    CraftItemsByHashObjectiveArgs, CraftItemsByHashObjectiveError, RequireModulesObjective,
};
use crate::utils::SortProductionCandidate as _;
use crate::utils::{
    ProductionCandidate, ProductionFromEnvironmentCandidate, ProductionFromIngredientsCandidate,
};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult,
};
use dudes_in_space_api::finance::{BankRegistry, Money};
use dudes_in_space_api::item::{ItemCount, ItemId, StorageRole};
use dudes_in_space_api::module::{AdminTradingConsole, ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonLogger, ThisPerson,
    tie,
};
use dudes_in_space_api::recipe::{InputItemRecipe, ItemRecipe, OutputItemRecipe};
use dudes_in_space_api::trade::OfferId;
use dudes_in_space_api::utils::math::NonNeg;
use dudes_in_space_api::utils::range::Range;
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::VesselInternalConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::{Deserialize, Deserializer, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter;
use std::rc::Rc;

/*
    - Find available crafts across all crafters
        - get list of all recipes of all crafters awailable to assemble
        - exclude recipes that are impossible to craft due to input resource missing on the market
        - find best offers for each remaining recipe
        - choose recipe with max profit
    - Craft chosen crafter
    - Place sell offer
    - Craft item
    - Place buy offer

    Example:
        There are selling offers for `steel` and `microelectronics` found.
        There is a buy offer for `microelectronics` found.
        No buy offers for `steel` are found.
        The `objective` prioritizes steel because nobody produces it yet.
        The `objective` checks if it can produce `steel`.
        Yes -> Goes for it.
        No -> Tries `microelectronics`

    Example:
        There are selling offers for `steel` and `microelectronics` found.
        There are buying offers for `steel` and `microelectronics` found.
        The `objective` checks if it can produce `steel` and `microelectronics`.
        Yes -> Chooses one that gives more profit.
        Can produce only `steal` -> Goes for it.

*/

static TYPE_ID: &str = "ManageProductionStationObjective";
static REQUIRED_CAPS: [ModuleCapability; 1] = [ModuleCapability::TradingTerminal];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "tp")]
#[deserialize_seed_xxx(seed = crate::objectives::management::manage_production_station_objective::StateSeed::<'context>)]
enum State {
    Idle,
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    DecideBestProduct {
        future: ReqFuture<FindBestOffersForItemsResult>,
        recipes_to_consider: BTreeSet<ItemRecipe>,
        input_recipes_to_consider: BTreeSet<InputItemRecipe>,
        output_recipes_to_consider: BTreeSet<OutputItemRecipe>,
    },
    RequireModules {
        objective: RequireModulesObjective,
        production_candidate: ProductionCandidate,
        output_limit: BTreeMap<ItemId, ItemCount>,
        input_limit: BTreeMap<ItemId, ItemCount>,
    },
    AssembleSpecificCrafter {
        craft_objective: CraftModulesByTypeIdObjective,
    },
    ExecuteProductionFromIngredients {
        production_candidate: ProductionFromIngredientsCandidate,
        input_limit: BTreeMap<ItemId, ItemCount>,
        craft_objective: CraftItemsByHashObjective,
        move_to_trading_terminal_objective: MoveToModuleObjective,
        sell_offers: BTreeMap<ItemId, OfferId>,
        buy_offers: BTreeMap<ItemId, OfferId>,
    },
    ExecuteProductionFromEnvironment {
        production_candidate: ProductionFromEnvironmentCandidate,
        output_objective: OutputItemsByHashObjective,
        move_to_trading_terminal_objective: MoveToModuleObjective,
        buy_offers: BTreeMap<ItemId, OfferId>,
    },
}

fn deserialize_forced_product<'de, D>(data: D) -> Result<Option<ItemId>, D::Error>
where
    D: Deserializer<'de>,
{
    let x = ItemId::deserialize(data).map(Some).unwrap_or(None);
    Ok(x)
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::objectives::management::manage_production_station_objective::ManageProductionStationObjectiveSeed::<'context>)]
pub(crate) struct ManageProductionStationObjective {
    #[deserialize_seed_xxx(seed = self.seed.state_seed)]
    state: State,
    #[serde(default, deserialize_with = "deserialize_forced_product")]
    forced_product: Option<ItemId>,
}

impl ManageProductionStationObjective {
    pub(crate) fn new(logger: &mut PersonLogger) -> Self {
        logger.info("ManageProductionStationObjective::new");
        Self {
            state: State::Idle,
            forced_product: None,
        }
    }
}

#[derive(Clone)]
struct StateSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindBestOffersForItemsResult>,
}

#[derive(Clone)]
struct ManageProductionStationObjectiveSeed<'context> {
    state_seed: StateSeed<'context>,
}

impl<'context> ManageProductionStationObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            state_seed: StateSeed {
                req_future_seed: ReqFutureSeed::new(context),
            },
        }
    }
}

impl Objective for ManageProductionStationObjective {
    type Result = ();
    type Error = ManageProductionStationObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match &mut self.state {
            State::Idle => {
                let item_recipes: BTreeSet<_> = iter::chain(
                    tie(this_module, this_vessel).item_recipes().into_iter(),
                    tie(this_module, this_vessel)
                        .potential_item_recipes()
                        .into_iter(),
                )
                .collect();

                let input_item_recipes: BTreeSet<_> = iter::chain(
                    tie(this_module, this_vessel)
                        .input_item_recipes()
                        .into_iter(),
                    tie(this_module, this_vessel)
                        .potential_input_item_recipes()
                        .into_iter(),
                )
                .collect();

                let output_item_recipes: BTreeSet<_> = iter::chain(
                    tie(this_module, this_vessel)
                        .output_item_recipes()
                        .into_iter(),
                    tie(this_module, this_vessel)
                        .potential_output_item_recipes()
                        .into_iter(),
                )
                .collect();

                let items: BTreeSet<_> = iter::chain(
                    input_item_recipes.iter().map(|x| x.items()).flatten(),
                    output_item_recipes.iter().map(|x| x.items()).flatten(),
                )
                .cloned()
                .collect();

                logger.info("ManageProductionStationObjective::FindBestOffersAndDecideBestRecipe");
                self.state = State::DecideBestProduct {
                    future: FindBestOffersForItems { items }
                        .push(environment_context.request_storage_mut()),
                    recipes_to_consider: item_recipes,
                    input_recipes_to_consider: input_item_recipes,
                    output_recipes_to_consider: output_item_recipes,
                };
                Ok(ObjectiveStatus::InProgress)
            }
            State::DecideBestProduct {
                future,
                recipes_to_consider,
                input_recipes_to_consider,
                output_recipes_to_consider,
            } => match future.take() {
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Ok(search_result) => {
                    if search_result.max_profit_sell_offers.is_empty() {
                        Err(Self::Error::NoSellOffersFound)
                    } else {
                        match choose_candidate(
                            &search_result,
                            &output_recipes_to_consider,
                            &recipes_to_consider,
                            environment_context.bank_registry(),
                            logger,
                            self.forced_product.clone(),
                        ) {
                            Some(ProductionCandidate::FromIngredients(production_candidate)) => {
                                self.forced_product = Some(production_candidate.product.clone());

                                let tied_vessel = tie(this_module, this_vessel);
                                match tied_vessel
                                    .find_item_recipe(production_candidate.recipe.hash())
                                {
                                    None => {
                                        let module_type_id = tied_vessel
                                            .potentially_craftable_modules_with_recipe(
                                                production_candidate.recipe.hash(),
                                            )
                                            .first()
                                            .unwrap()
                                            .clone();

                                        logger.info("ManageProductionStationObjective::AssembleSpecificCrafter (Prod from ingredients)");
                                        self.state = State::AssembleSpecificCrafter {
                                            craft_objective: CraftModulesByTypeIdObjective::new(
                                                CraftModulesByTypeIdObjectiveArgs {
                                                    modules: vec![module_type_id],
                                                    deploy: true,
                                                    wait_if_has_no_ingredients: false,
                                                },
                                                logger,
                                            ),
                                        };
                                        Ok(ObjectiveStatus::InProgress)
                                    }
                                    Some((module, recipe)) => {
                                        let input_storages =
                                            module.storages_by_role(StorageRole::Input);
                                        let output_storages =
                                            module.storages_by_role(StorageRole::Output);

                                        let input_storage = input_storages.first().unwrap();
                                        let output_storage = output_storages.first().unwrap();

                                        let single_input_capacity =
                                            input_storage.capacity() / recipe.input.len();
                                        let single_output_capacity =
                                            output_storage.capacity() / recipe.output.len();

                                        let mut input_limit: BTreeMap<ItemId, ItemCount> =
                                            Default::default();
                                        for stack in recipe.input {
                                            let item = environment_context
                                                .item_vault()
                                                .get_ref(stack.id)
                                                .unwrap();

                                            let item_max_count =
                                                (single_input_capacity / item.volume).ceil()
                                                    as ItemCount;

                                            assert!(stack.count <= item_max_count);
                                            input_limit
                                                .try_insert(item.id.clone(), item_max_count)
                                                .unwrap();
                                        }

                                        let mut output_limit: BTreeMap<ItemId, ItemCount> =
                                            Default::default();
                                        for (item, recipe_count) in recipe.output {
                                            let item = environment_context
                                                .item_vault()
                                                .get_ref(item)
                                                .unwrap();
                                            let item_max_count =
                                                (single_output_capacity / item.volume).ceil()
                                                    as ItemCount;
                                            assert!(recipe_count <= item_max_count);
                                            output_limit
                                                .try_insert(item.id.clone(), item_max_count)
                                                .unwrap();
                                        }

                                        let terminals = tied_vessel.modules_with_capability(
                                            ModuleCapability::TradingTerminal,
                                        );

                                        logger.info(
                                            "ManageProductionStationObjective::RequireModules (Prod from ingredients)",
                                        );
                                        self.state = State::RequireModules {
                                            input_limit,
                                            output_limit,
                                            production_candidate:
                                                ProductionCandidate::FromIngredients(
                                                    production_candidate,
                                                ),
                                            objective: RequireModulesObjective::new(
                                                REQUIRED_CAPS.into(),
                                                [].into(),
                                                logger,
                                            ),
                                        };
                                        Ok(ObjectiveStatus::InProgress)
                                    }
                                }
                            }
                            Some(ProductionCandidate::FromEnvironment(production_candidate)) => {
                                self.forced_product = Some(production_candidate.product.clone());

                                let tied_vessel = tie(this_module, this_vessel);
                                match tied_vessel
                                    .find_output_item_recipe(production_candidate.recipe.hash())
                                {
                                    None => {
                                        let module_type_id = tied_vessel
                                            .potentially_craftable_modules_with_output_recipe(
                                                production_candidate.recipe.hash(),
                                            )
                                            .first()
                                            .unwrap()
                                            .clone();

                                        logger.info("ManageProductionStationObjective::AssembleSpecificCrafter (Prod from env)");
                                        self.state = State::AssembleSpecificCrafter {
                                            craft_objective: CraftModulesByTypeIdObjective::new(
                                                CraftModulesByTypeIdObjectiveArgs {
                                                    modules: vec![module_type_id],
                                                    deploy: true,
                                                    wait_if_has_no_ingredients: false,
                                                },
                                                logger,
                                            ),
                                        };
                                        Ok(ObjectiveStatus::InProgress)
                                    }
                                    Some((module, recipe)) => {
                                        let output_storages =
                                            module.storages_by_role(StorageRole::Output);

                                        let output_storage = output_storages.first().unwrap();

                                        let single_output_capacity =
                                            output_storage.capacity() / recipe.len();

                                        let mut output_limit: BTreeMap<ItemId, ItemCount> =
                                            Default::default();
                                        for (item, recipe_count) in recipe {
                                            let item = environment_context
                                                .item_vault()
                                                .get_ref(item)
                                                .unwrap();
                                            let item_max_count =
                                                (single_output_capacity / item.volume).ceil()
                                                    as ItemCount;
                                            assert!(recipe_count <= item_max_count);
                                            output_limit
                                                .try_insert(item.id.clone(), item_max_count)
                                                .unwrap();
                                        }

                                        let terminals = tied_vessel.modules_with_capability(
                                            ModuleCapability::TradingTerminal,
                                        );
                                        logger.info(
                                            "ManageProductionStationObjective::RequireModules (Prod from env)",
                                        );
                                        self.state = State::RequireModules {
                                            input_limit: BTreeMap::new(),
                                            output_limit,
                                            production_candidate:
                                                ProductionCandidate::FromEnvironment(
                                                    production_candidate,
                                                ),
                                            objective: RequireModulesObjective::new(
                                                REQUIRED_CAPS.into(),
                                                [].into(),
                                                logger,
                                            ),
                                        };
                                        Ok(ObjectiveStatus::InProgress)
                                    }
                                }
                            }
                            None => todo!(),
                        }
                    }
                }
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            State::RequireModules {
                objective,
                production_candidate,
                output_limit,
                input_limit,
            } => match objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done(_)) => {
                    logger.info("Checking all prerequisites to managing production station...");

                    let tied_vessel = tie(this_module, this_vessel);

                    let terminals =
                        tied_vessel.modules_with_capability(ModuleCapability::TradingTerminal);

                    match production_candidate {
                        ProductionCandidate::FromIngredients(production_candidate) => {
                            logger.info("ManageProductionStationObjective::ExecuteProductionFromIngredients");
                            self.state = State::ExecuteProductionFromIngredients {
                                input_limit: std::mem::take(input_limit),
                                craft_objective: CraftItemsByHashObjective::new(
                                    CraftItemsByHashObjectiveArgs {
                                        recipe_hash: production_candidate.recipe.hash(),
                                        output_limit: std::mem::take(output_limit),
                                        done_if_reached_limit: false,
                                        behaviour_if_lack_ingredients:
                                            BehaviourIfLackIngredients::WaitFor { cycles: 20 },
                                        interrupt_after_each_craft: true,
                                        start_interrupted: true,
                                    },
                                    logger,
                                ),
                                production_candidate: production_candidate.clone(),
                                move_to_trading_terminal_objective: MoveToModuleObjective::new(
                                    terminals.first().unwrap().id(),
                                ),
                                buy_offers: BTreeMap::new(),
                                sell_offers: BTreeMap::new(),
                            };
                            Ok(ObjectiveStatus::InProgress)
                        }
                        ProductionCandidate::FromEnvironment(production_candidate) => {
                            logger.info("ManageProductionStationObjective::ExecuteProductionFromEnvironment");
                            self.state = State::ExecuteProductionFromEnvironment {
                                output_objective: OutputItemsByHashObjective::new(
                                    OutputItemsByHashObjectiveArgs {
                                        recipe_hash: production_candidate.recipe.hash(),
                                        output_limit: std::mem::take(output_limit),
                                        done_if_reached_limit: false,
                                        interrupt_after_each_craft: true,
                                        start_interrupted: true,
                                    },
                                    logger,
                                ),
                                production_candidate: production_candidate.clone(),
                                move_to_trading_terminal_objective: MoveToModuleObjective::new(
                                    terminals.first().unwrap().id(),
                                ),
                                buy_offers: BTreeMap::new(),
                            };
                            Ok(ObjectiveStatus::InProgress)
                        }
                    }
                }
                Err(err) => Err(Self::Error::CraftingOtherModulesError(err)),
            },
            State::AssembleSpecificCrafter { craft_objective } => match craft_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done(_)) => {
                    logger.info("collect all available recipes to managing production station...");
                    self.state = State::Idle;
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(err) => Err(Self::Error::CraftingFabricatorError(err)),
            },
            State::ExecuteProductionFromIngredients {
                production_candidate,
                input_limit,
                craft_objective,
                move_to_trading_terminal_objective,
                buy_offers,
                sell_offers,
            } => {
                match craft_objective.pursue(
                    this_person,
                    this_module,
                    this_vessel,
                    environment_context,
                    logger,
                ) {
                    Ok(ObjectiveStatus::InProgress) => {
                        if craft_objective.is_interrupted() {
                            match move_to_trading_terminal_objective.pursue(
                                this_person,
                                this_module,
                                this_vessel,
                                environment_context,
                                logger,
                            ) {
                                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                                Ok(ObjectiveStatus::Done(_)) => {
                                    let this_vessel = tie(this_module, this_vessel);

                                    let crafting_module = this_vessel
                                        .module_by_id(craft_objective.crafting_module().unwrap())
                                        .unwrap();

                                    let input_storages =
                                        crafting_module.storages_by_role(StorageRole::Input);
                                    let output_storages =
                                        crafting_module.storages_by_role(StorageRole::Output);

                                    let input_storage = input_storages.first().unwrap();
                                    let output_storage = output_storages.first().unwrap();

                                    let input_needed =
                                        input_storage.content().lack(input_limit.clone());
                                    let output_has_in_storage = output_storage.content().counts(
                                        production_candidate.recipe.output.items().cloned(),
                                    );

                                    let mut offer_update_instructions: Vec<OfferUpdateInstruction> = input_needed
                                        .iter()
                                        .map(|(item, count)| {
                                            assert_ne!(*count, 0);

                                            OfferUpdateInstruction {
                                                kind: OfferUpdateInstructionKind::Sell,
                                                id: sell_offers.get(item).cloned(),
                                                item: item.clone(),
                                                count_range: (1..*count).into(),
                                                price_per_unit: production_candidate
                                                    .average_ingredients_buy_price
                                                    .get(item)
                                                    .cloned()
                                                    .unwrap_or_else(||
                                                        Money {
                                                            currency: this_person.preferred_currency_or_create_default(
                                                                environment_context,
                                                            ),
                                                            amount: NonNeg::new( 1).unwrap(),
                                                        }
                                                    )
                                            }
                                        })
                                        .collect();

                                    offer_update_instructions.push(OfferUpdateInstruction {
                                        kind: OfferUpdateInstructionKind::Buy,
                                        id: buy_offers.get(&production_candidate.product).cloned(),
                                        item: production_candidate.product.clone(),
                                        count_range: (1..output_has_in_storage
                                            .get(&production_candidate.product)
                                            .unwrap()
                                            .clone())
                                            .into(),
                                        price_per_unit: production_candidate
                                            .average_product_sell_price
                                            .clone()
                                            * this_person.notes.margin(),
                                    });

                                    drop(crafting_module);
                                    place_or_update_offers(
                                        this_module.trading_admin_console_mut().unwrap(),
                                        offer_update_instructions,
                                        sell_offers,
                                        buy_offers,
                                    );

                                    craft_objective.resume();
                                    Ok(ObjectiveStatus::InProgress)
                                }
                                Err(err) => todo!("{:?}", err),
                            }
                        } else {
                            Ok(ObjectiveStatus::InProgress)
                        }
                    }
                    Ok(ObjectiveStatus::Done(result)) => todo!("result: {:?}", result),
                    Err(CraftItemsByHashObjectiveError::CanNotFindCraftingModule) => {
                        logger.info("ManageProductionStationObjective::RequireModules");
                        self.state = State::RequireModules {
                            input_limit: std::mem::take(input_limit),
                            output_limit: craft_objective.args().output_limit.clone(),
                            production_candidate: ProductionCandidate::FromIngredients(
                                production_candidate.clone(),
                            ),
                            objective: RequireModulesObjective::new(
                                REQUIRED_CAPS.into(),
                                [].into(),
                                logger,
                            ),
                        };
                        Ok(ObjectiveStatus::InProgress)
                    }
                    Err(CraftItemsByHashObjectiveError::LackIngredients) => {
                        self.state = State::Idle;
                        Ok(ObjectiveStatus::InProgress)
                    }
                }
            }
            State::ExecuteProductionFromEnvironment {
                production_candidate,
                output_objective,
                move_to_trading_terminal_objective,
                buy_offers,
            } => {
                match output_objective.pursue(
                    this_person,
                    this_module,
                    this_vessel,
                    environment_context,
                    logger,
                ) {
                    Ok(ObjectiveStatus::InProgress) => {
                        if output_objective.is_interrupted() {
                            match move_to_trading_terminal_objective.pursue(
                                this_person,
                                this_module,
                                this_vessel,
                                environment_context,
                                logger,
                            ) {
                                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                                Ok(ObjectiveStatus::Done(_)) => {
                                    let this_vessel = tie(this_module, this_vessel);

                                    let crafting_module = this_vessel
                                        .module_by_id(output_objective.crafting_module().unwrap())
                                        .unwrap();

                                    let output_storages =
                                        crafting_module.storages_by_role(StorageRole::Output);

                                    let output_storage = output_storages.first().unwrap();

                                    let output_has_in_storage = output_storage
                                        .content()
                                        .counts(production_candidate.recipe.items().cloned());

                                    let offer_update_instructions = vec![OfferUpdateInstruction {
                                        kind: OfferUpdateInstructionKind::Buy,
                                        id: buy_offers.get(&production_candidate.product).cloned(),
                                        item: production_candidate.product.clone(),
                                        count_range: (1..output_has_in_storage
                                            .get(&production_candidate.product)
                                            .unwrap()
                                            .clone())
                                            .into(),
                                        price_per_unit: production_candidate
                                            .average_product_sell_price
                                            .clone()
                                            * this_person.notes.margin(),
                                    }];

                                    drop(crafting_module);
                                    place_or_update_offers(
                                        this_module.trading_admin_console_mut().unwrap(),
                                        offer_update_instructions,
                                        &mut BTreeMap::new(),
                                        buy_offers,
                                    );

                                    output_objective.resume();
                                    Ok(ObjectiveStatus::InProgress)
                                }
                                Err(err) => todo!("{:?}", err),
                            }
                        } else {
                            Ok(ObjectiveStatus::InProgress)
                        }
                    }
                    Ok(ObjectiveStatus::Done(result)) => todo!("result: {:?}", result),
                    Err(OutputItemsByHashObjectiveError::CanNotFindCraftingModule) => todo!(),
                }
            }
        }
    }
}

pub(crate) struct ManageProductionStationObjectiveDecider;

impl ObjectiveDecider for ManageProductionStationObjectiveDecider {
    fn consider(
        &self,
        person: &ThisPerson,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if (person.passions.contains(&Passion::Management)
            || person.passions.contains(&Passion::Ruling))
            && person.passions.contains(&Passion::Crafting)
        {
            logger.info("Manage production station objective decided.");
            Some(Box::new(ManageProductionStationObjective::new(logger)))
        } else {
            None
        }
    }
}

impl DynSerialize for ManageProductionStationObjective {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct ManageProductionStationObjectiveDynSeed {
    req_context: Rc<ReqContext>,
}

impl ManageProductionStationObjectiveDynSeed {
    pub fn new(req_context: Rc<ReqContext>) -> Self {
        Self { req_context }
    }
}

impl DynDeserializeSeed<dyn DynObjective> for ManageProductionStationObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let obj: ManageProductionStationObjective = from_intermediate_seed(
            ManageProductionStationObjectiveSeed::new(&self.req_context),
            &intermediate,
        )
        .map_err(Box::new)?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug)]
pub(crate) enum ManageProductionStationObjectiveError {
    CraftingFabricatorError(CraftModulesByTypeIdObjectiveError),
    CraftingOtherModulesError(CraftModulesObjectiveError),
    NoSellOffersFound,
}

impl Display for ManageProductionStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ManageProductionStationObjectiveError {}

fn choose_candidate(
    search_result: &FindBestOffersForItemsResult,
    output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
    recipes_to_consider: &BTreeSet<ItemRecipe>,
    bank_registry: &BankRegistry,
    logger: &mut PersonLogger,
    preferred_product: Option<ItemId>,
) -> Option<ProductionCandidate> {
    let mut production_candidates = ProductionCandidate::build_for_items_in_demand(
        search_result,
        recipes_to_consider,
        output_recipes_to_consider,
        bank_registry,
    );
    if !production_candidates.is_empty() {
        logger.info(format!(
            "Production candidates ({}):",
            production_candidates.len()
        ));

        for i in &production_candidates {
            logger.info(format!("\t{}", i));
        }

        production_candidates.sort(bank_registry, preferred_product.clone());

        if let Some(preferred_product) = &preferred_product {
            logger.info(format!(
                "Sorted production candidates (Preferred product: {}) ({}):",
                production_candidates.len(),
                preferred_product
            ));
        } else {
            logger.info(format!(
                "Sorted production candidates ({}):",
                production_candidates.len()
            ));
        }

        for production_candidate in &production_candidates {
            logger.info(format!("\t{}", production_candidate));
        }

        let best_candidate = production_candidates.first().unwrap();
        Some(best_candidate.clone())
    } else {
        todo!(
            "Find item with best profit setting buy price as cheap as possible while sustaining margin"
        )
    }
}

enum OfferUpdateInstructionKind {
    Buy,
    Sell,
}

struct OfferUpdateInstruction {
    pub kind: OfferUpdateInstructionKind,
    pub id: Option<OfferId>,
    pub item: ItemId,
    pub count_range: Range<ItemCount>,
    pub price_per_unit: Money,
}

fn place_or_update_offers(
    trading_console: &mut dyn AdminTradingConsole,
    instructions: Vec<OfferUpdateInstruction>,
    sell_offers: &mut BTreeMap<ItemId, OfferId>,
    buy_offers: &mut BTreeMap<ItemId, OfferId>,
) {
    for instruction in instructions {
        match instruction.kind {
            OfferUpdateInstructionKind::Buy => {
                if let Some(id) = instruction.id {
                    trading_console
                        .update_buy_offer(
                            id,
                            instruction.item,
                            instruction.count_range,
                            instruction.price_per_unit,
                        )
                        .unwrap();
                } else {
                    let offer = trading_console
                        .place_buy_offer(
                            instruction.item,
                            instruction.count_range,
                            instruction.price_per_unit,
                        )
                        .unwrap();
                    buy_offers.try_insert(offer.item.clone(), offer.id).unwrap();
                }
            }
            OfferUpdateInstructionKind::Sell => {
                if let Some(id) = instruction.id {
                    trading_console
                        .update_sell_offer(
                            id,
                            instruction.item,
                            instruction.count_range,
                            instruction.price_per_unit,
                        )
                        .unwrap();
                } else {
                    let offer = trading_console
                        .place_sell_offer(
                            instruction.item,
                            instruction.count_range,
                            instruction.price_per_unit,
                        )
                        .unwrap();
                    sell_offers
                        .try_insert(offer.item.clone(), offer.id)
                        .unwrap();
                }
            }
        }
    }
}

impl Display for ManageProductionStationObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.state {
            State::Idle => write!(f, "Idle"),
            State::DecideBestProduct { .. } => {
                write!(f, "DecideBestProduct")
            }
            State::RequireModules { objective, .. } => write!(f, "RequireModules -> {}", objective),
            State::AssembleSpecificCrafter { craft_objective } => {
                write!(f, "RequireModules -> {}", craft_objective)
            }
            State::ExecuteProductionFromIngredients {
                craft_objective, ..
            } => write!(f, "ExecuteProductionFromIngredients -> {}", craft_objective),
            State::ExecuteProductionFromEnvironment {
                output_objective, ..
            } => write!(
                f,
                "ExecuteProductionFromEnvironment -> {}",
                output_objective
            ),
        }
    }
}
