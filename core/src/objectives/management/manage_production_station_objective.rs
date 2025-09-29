use crate::objectives::common::MoveToModuleObjective;
use crate::objectives::crafting::{CraftItemsByHashObjective, CraftModulesObjectiveArgs, CraftModulesObjectiveError, OutputItemsByHashObjective};
use crate::objectives::crafting::{
    CraftItemsByHashObjectiveArgs, CraftItemsByHashObjectiveError, CraftModulesObjective,
    RequireModulesObjective,
};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult,
};
use dudes_in_space_api::finance::Money;
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
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::cmp::Ordering;
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
static REQUIRED_CAPS: [ModuleCapability; 2] = [
    ModuleCapability::ItemCrafting,
    ModuleCapability::TradingTerminal,
];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "manage_production_station_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::management::manage_production_station_objective::ManageProductionStationObjectiveSeed::<'context>)]
pub(crate) enum ManageProductionStationObjective {
    CollectAllAvailableRecipes,
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    FindBestOffersAndDecideBestRecipe {
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
        craft_objective: CraftModulesObjective,
    },
    ExecuteProductionFromIngredients {
        production_candidate: ProductionFromIngredientsCandidate,
        input_limit: BTreeMap<ItemId, ItemCount>,
        craft_objective: CraftItemsByHashObjective,
        move_to_trading_terminal_objective: MoveToModuleObjective,
        /// TODO: save placed offers here so u can know what to update
        sell_offers: BTreeMap<ItemId, OfferId>,
        /// TODO: save placed offers here so u can know what to update
        buy_offers: BTreeMap<ItemId, OfferId>,
    },
    ExecuteProductionFromEnvironment {
        production_candidate: ProductionFromEnvironmentCandidate,
        output_objective: OutputItemsByHashObjective,
        move_to_trading_terminal_objective: MoveToModuleObjective,
        /// TODO: save placed offers here so u can know what to update
        sell_offers: BTreeMap<ItemId, OfferId>,
    },
}

impl ManageProductionStationObjective {
    pub(crate) fn new(logger: &mut PersonLogger) -> Self {
        logger.info("ManageProductionStationObjective::new");
        Self::CollectAllAvailableRecipes
    }
}

struct ManageProductionStationObjectiveSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindBestOffersForItemsResult>,
}

impl<'context> ManageProductionStationObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            req_future_seed: ReqFutureSeed::new(context),
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
        match self {
            Self::CollectAllAvailableRecipes => {
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
                *self = Self::FindBestOffersAndDecideBestRecipe {
                    future: FindBestOffersForItems { items }
                        .push(environment_context.request_storage_mut()),
                    recipes_to_consider: item_recipes,
                    input_recipes_to_consider: input_item_recipes,
                    output_recipes_to_consider: output_item_recipes,
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::FindBestOffersAndDecideBestRecipe {
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
                        // println!("recipes_to_consider: {:#?}", recipes_to_consider,);
                        // println!(
                        //     "input_recipes_to_consider: {:#?}",
                        //     input_recipes_to_consider,
                        // );
                        //
                        // println!(
                        //     "output_recipes_to_consider: {:#?}",
                        //     output_recipes_to_consider,
                        // );

                        match choose_item_to_produce(
                            &search_result,
                            &output_recipes_to_consider,
                            &recipes_to_consider,
                        ) {
                            Some(ProductionCandidate::FromIngredients(production_candidate)) => {
                                let tied_vessel = tie(this_module, this_vessel);
                                match tied_vessel
                                    .find_item_recipe(production_candidate.recipe.hash())
                                {
                                    None => {
                                        // TODO craft specific module for this recipe
                                        *self = Self::AssembleSpecificCrafter {
                                            craft_objective: CraftModulesObjective::new(
                                                [ModuleCapability::ItemCrafting].into(),
                                                [].into(),
                                                CraftModulesObjectiveArgs {
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

                                        *self = Self::RequireModules {
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
                                todo!()
                            }
                            None => todo!(),
                        }
                    }
                }
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::RequireModules {
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
                            *self = Self::ExecuteProductionFromIngredients {
                                input_limit: std::mem::take(input_limit),
                                craft_objective: CraftItemsByHashObjective::new(
                                    CraftItemsByHashObjectiveArgs {
                                        recipe_hash: production_candidate.recipe.hash(),
                                        output_limit: std::mem::take(output_limit),
                                        done_if_reached_limit: false,
                                        err_if_lack_ingredients: false,
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
                            todo!()
                        }
                    }
                }
                Err(err) => Err(Self::Error::CraftingFabricatorError(err)),
            },
            Self::AssembleSpecificCrafter { craft_objective } => match craft_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done(_)) => {
                    logger.info("collect all available recipes to managing production station...");
                    *self = Self::CollectAllAvailableRecipes;
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(err) => Err(Self::Error::CraftingFabricatorError(err)),
            },
            Self::ExecuteProductionFromIngredients {
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
                                    let output_needed = output_storage
                                        .content()
                                        .lack(craft_objective.args().output_limit.clone());

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
                                        count_range: (1..output_needed
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
                        *self = Self::RequireModules {
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
                        unreachable!("Cuz err_if_lack_ingredients is set to false")
                    }
                }
            }
            Self::ExecuteProductionFromEnvironment { .. } => todo!()
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
        if person.passions.contains(&Passion::Management)
            || person.passions.contains(&Passion::Ruling)
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
    CraftingFabricatorError(CraftModulesObjectiveError),
    NoSellOffersFound,
}

impl Display for ManageProductionStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ManageProductionStationObjectiveError {}

impl Display for ManageProductionStationObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CollectAllAvailableRecipes => write!(f, "CollectAllAvailableRecipes"),
            Self::FindBestOffersAndDecideBestRecipe { .. } => {
                write!(f, "FindBestOffersAndDecideBestRecipe")
            }
            Self::RequireModules { objective, .. } => write!(f, "RequireModules -> {}", objective),
            Self::AssembleSpecificCrafter { craft_objective } => {
                write!(f, "RequireModules -> {}", craft_objective)
            }
            Self::ExecuteProductionFromIngredients {
                craft_objective, ..
            } => write!(f, "ExecuteProductionFromIngredients -> {}", craft_objective),
            Self::ExecuteProductionFromEnvironment {
                output_objective, ..
            } => write!(f, "ExecuteProductionFromEnvironment -> {}", output_objective),
        }
    }
}

fn calc_items_in_demand(
    search_result: &FindBestOffersForItemsResult,
    output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
) -> BTreeMap<ItemId, Money> {
    search_result
        .average_sell_offers
        .clone()
        .into_iter()
        .filter(|(item, _)| {
            output_recipes_to_consider
                .iter()
                .find(|recipe| {
                    recipe
                        .items()
                        .find(|recipe_item| *recipe_item == item)
                        .is_some()
                })
                .is_some()
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProductionCandidateEstimate {
    /// Price of all ingredients needed to produce one unit of product.
    cost_price: Money,
    profit: Money,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProductionFromIngredientsCandidate {
    product: ItemId,
    average_product_sell_price: Money,
    average_ingredients_buy_price: BTreeMap<ItemId, Money>,
    has_all_ingredients_on_market: bool,
    has_producers_on_market: bool,
    recipe: ItemRecipe,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    estimate: Option<ProductionCandidateEstimate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProductionFromEnvironmentCandidate {
    product: ItemId,
    average_product_sell_price: Money,
    average_ingredients_buy_price: BTreeMap<ItemId, Money>,
    has_all_ingredients_on_market: bool,
    has_producers_on_market: bool,
    recipe: ItemRecipe,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    estimate: Option<ProductionCandidateEstimate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "tp")]
pub(crate) enum ProductionCandidate {
    FromIngredients(ProductionFromIngredientsCandidate),
    FromEnvironment(ProductionFromEnvironmentCandidate),
}

impl Display for ProductionCandidateEstimate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for ProductionFromIngredientsCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.estimate {
            None => write!(
                f,
                "p: {}, avr_prod_sell_price: {}, has_all_ing_on_m: {}, has_producers_on_m: {}, r: {}",
                self.product,
                self.average_product_sell_price,
                self.has_all_ingredients_on_market,
                self.has_producers_on_market,
                self.recipe,
            ),
            Some(estimate) => write!(
                f,
                "p: {}, avr_prod_sell_price: {}, has_all_ing_on_m: {}, has_producers_on_m: {}, r: {}, est: {}",
                self.product,
                self.average_product_sell_price,
                self.has_all_ingredients_on_market,
                self.recipe,
                self.has_producers_on_market,
                estimate,
            ),
        }
    }
}

impl Display for ProductionFromEnvironmentCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for ProductionCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ProductionCandidate {
    fn build_vec(
        search_result: &FindBestOffersForItemsResult,
        items_to_consider: BTreeMap<ItemId, Money>,
        recipes_to_consider: &BTreeSet<ItemRecipe>,
        output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
    ) -> Vec<Self> {
        items_to_consider
            .into_iter()
            .filter_map(|(item, _)| {
                let has_producers_on_market = search_result
                    .average_buy_offers
                    .iter()
                    .find(|(offer_item, _)| *offer_item == &item)
                    .is_some();

                println!("recipes_to_consider: {:?}", recipes_to_consider);

                match recipes_to_consider.iter().find(|recipe| {
                    recipe
                        .output
                        .items()
                        .find(|recipe_item| *recipe_item == &item)
                        .is_some()
                }) {
                    None => {
                        println!(
                            "output_recipes_to_consider: {:?}",
                            output_recipes_to_consider
                        );

                        match output_recipes_to_consider.iter().find(|recipe| {
                            recipe
                                .items()
                                .find(|recipe_item| *recipe_item == &item)
                                .is_some()
                        }) {
                            None => {
                                todo!()
                            }
                            Some(recipe) => {
                                // Should check if the output recipe requires some action to produce
                                // if not return true
                                // if yes, check if it can do this action (for example, mine asteroids or collect gas from nebula, or some other natural occurring resource)
                                todo!("has only an output recipe: {:?}", recipe)
                            }
                        }
                    }
                    Some(recipe) => {
                        assert!(!recipe.input.is_empty());

                        let mut has_all_ingredients_on_market: bool = true;
                        let mut average_ingredients_buy_price: BTreeMap<ItemId, Money> =
                            Default::default();
                        let mut sum_ingredients_cost_price: Option<Money> = None;
                        for (item, count) in &recipe.input {
                            if let Some((_, average_buy_price)) = search_result
                                .average_buy_offers
                                .iter()
                                .find(|(offer_item, _)| *offer_item == item)
                            {
                                average_ingredients_buy_price
                                    .try_insert(item.clone(), average_buy_price.clone())
                                    .unwrap();

                                let stack_price = average_buy_price.clone() * *count;
                                match &mut sum_ingredients_cost_price {
                                    None => sum_ingredients_cost_price = Some(stack_price),
                                    Some(sum_ingredients_cost_price) => sum_ingredients_cost_price
                                        .add_assign_same_currency(stack_price)
                                        .unwrap(),
                                }
                            } else {
                                has_all_ingredients_on_market = false;
                            }
                        }

                        if let Some((_, average_product_sell_price)) = search_result
                            .average_sell_offers
                            .iter()
                            .find(|(offer_item, _)| *offer_item == &item)
                        {
                            Some(Self::FromIngredients(ProductionFromIngredientsCandidate {
                                product: item.clone(),
                                average_product_sell_price: average_product_sell_price.clone(),
                                average_ingredients_buy_price,
                                has_all_ingredients_on_market,
                                has_producers_on_market,
                                recipe: recipe.clone(),
                                estimate: sum_ingredients_cost_price.map(
                                    |sum_ingredients_cost_price| ProductionCandidateEstimate {
                                        cost_price: sum_ingredients_cost_price.clone(),
                                        profit: average_product_sell_price
                                            .clone()
                                            .sub_same_currency(sum_ingredients_cost_price)
                                            .unwrap(),
                                    },
                                ),
                            }))
                        } else {
                            None
                        }
                    }
                }
            })
            .collect()
    }
}

fn cmp_option<T, F: FnOnce(T, T) -> Ordering>(a: Option<T>, b: Option<T>, f: F) -> Ordering {
    let a_is_none = a.is_none();
    if let (Some(a), Some(b)) = (a, b) {
        f(a, b)
    } else if a_is_none {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

fn choose_item_to_produce(
    search_result: &FindBestOffersForItemsResult,
    output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
    recipes_to_consider: &BTreeSet<ItemRecipe>,
) -> Option<ProductionCandidate> {
    let items_in_demand = calc_items_in_demand(&search_result, &output_recipes_to_consider);

    if !items_in_demand.is_empty() {
        items_in_demand.iter().for_each(|(item, _)| {
            println!("items_in_demand: {:?}", item);
        });

        let mut production_candidates = ProductionCandidate::build_vec(
            search_result,
            items_in_demand,
            recipes_to_consider,
            output_recipes_to_consider,
        );
        if !production_candidates.is_empty() {
            for i in &production_candidates {
                println!("production_candidates: {}", i);
            }

            production_candidates.sort_by(|a, b| match (a, b) {
                (
                    ProductionCandidate::FromIngredients(a),
                    ProductionCandidate::FromIngredients(b),
                ) => a
                    .has_producers_on_market
                    .cmp(&b.has_producers_on_market)
                    .then(
                        a.has_all_ingredients_on_market
                            .cmp(&b.has_all_ingredients_on_market),
                    )
                    .then_with(|| {
                        cmp_option(a.estimate.as_ref(), b.estimate.as_ref(), |a, b| {
                            a.profit.cmp_same_currency(&b.profit).unwrap()
                        })
                    })
                    .then(
                        a.average_ingredients_buy_price
                            .len()
                            .cmp(&b.average_ingredients_buy_price.len()),
                    ),
                (
                    ProductionCandidate::FromEnvironment(a),
                    ProductionCandidate::FromEnvironment(b),
                ) => todo!(),
                (
                    ProductionCandidate::FromIngredients(a),
                    ProductionCandidate::FromEnvironment(b),
                ) => Ordering::Less,
                (
                    ProductionCandidate::FromEnvironment(a),
                    ProductionCandidate::FromIngredients(b),
                ) => Ordering::Equal,
            });

            for i in &production_candidates {
                println!("sorted production_candidates: {}", i);
            }

            let best_candidate = production_candidates.first().unwrap();
            Some(best_candidate.clone())
        } else {
            todo!(
                "Find item with best profit setting buy price as cheap as possible while sustaining margin"
            )
        }
    } else {
        todo!("Has no items in demand. wait until some items are needed")
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
