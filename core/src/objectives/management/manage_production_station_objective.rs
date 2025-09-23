use crate::objectives::crafting::CraftItemsByHashObjectiveArgs;
use crate::objectives::crafting::{
    CraftItemsByHashObjective, CraftModulesObjective, CraftModulesObjectiveError,
};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult,
};
use dudes_in_space_api::finance::Money;
use dudes_in_space_api::item::{ ItemId};
use dudes_in_space_api::module::{ModuleConsole};
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonLogger, ThisPerson,
    tie,
};
use dudes_in_space_api::recipe::{InputItemRecipe, ItemRecipe, ItemRecipeHash, OutputItemRecipe};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::VesselInternalConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
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
    AssembleCrafter {
        craft_objective: CraftModulesObjective,
    },
    ExecuteProduction {
        craft_objective: CraftItemsByHashObjective,
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
                            ChooseItemToProduceResult::Craft {
                                item,
                                recipe,
                            } => {

                                match tie(
                                    this_module,
                                    this_vessel,
                                ).find_item_recipe(recipe) {
                                    None => todo!("Assemble crafter"),
                                    Some((module, recipe )) => {

                                        let items_limit = (||todo!("Extract output capacity from crafter module"))();

                                        *self = Self::ExecuteProduction {
                                            craft_objective: CraftItemsByHashObjective::new(
                                                CraftItemsByHashObjectiveArgs {
                                                    recipe: recipe.default_hash(),
                                                    items_limit,
                                                    done_if_reached_limit: false,
                                                    err_if_lack_ingredients: false,
                                                },
                                                logger,
                                            ),
                                        };
                                        Ok(ObjectiveStatus::InProgress)
                                    }
                                }
                            }
                            ChooseItemToProduceResult::ProduceFromEnvironment { .. } => todo!(),
                            ChooseItemToProduceResult::NotFound => todo!(),
                        }
                    }
                }
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::ExecuteProduction { craft_objective } => {
                match craft_objective.pursue(
                    this_person,
                    this_module,
                    this_vessel,
                    environment_context,
                    logger,
                ) {
                    Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                    Ok(ObjectiveStatus::Done(result)) => todo!("result: {:?}", result),
                    Err(err) => todo!("err: {:?}", err),
                }
            }
            Self::AssembleCrafter { craft_objective } => match craft_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done(_)) => {
                    logger.info("Checking all prerequisites to managing production station...");

                    let recipe = (|| todo!())();
                    let items_limit = (|| todo!())();

                    *self = Self::ExecuteProduction {
                        craft_objective: CraftItemsByHashObjective::new(
                            CraftItemsByHashObjectiveArgs {
                                recipe,
                                items_limit,
                                done_if_reached_limit: false,
                                err_if_lack_ingredients: false,
                            },
                            logger,
                        ),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(err) => Err(Self::Error::CraftingFabricatorError(err)),
            },
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
            Self::AssembleCrafter { .. } => write!(f, "AssembleCrafter"),
            Self::ExecuteProduction { .. } => write!(f, "ExecuteProduction"),
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

fn calc_items_in_demand_that_no_one_produces(
    search_result: &FindBestOffersForItemsResult,
    items_in_demand: BTreeMap<ItemId, Money>,
) -> BTreeMap<ItemId, Money> {
    items_in_demand
        .into_iter()
        .filter(|(item, _)| {
            search_result
                .average_buy_offers
                .iter()
                .find(|(offer_item, _)| *offer_item == item)
                .is_none()
        })
        .collect()
}

fn calc_items_in_demand_that_no_one_produces_and_ingredients_are_on_market(
    search_result: &FindBestOffersForItemsResult,
    items_in_demand_that_no_one_produces: BTreeMap<ItemId, Money>,
    recipes_to_consider: &BTreeSet<ItemRecipe>,
) -> BTreeMap<ItemId, Money> {
    items_in_demand_that_no_one_produces
        .into_iter()
        .filter(|(item, _)| {
            match recipes_to_consider.iter().find(|recipe| {
                recipe
                    .output
                    .items()
                    .find(|recipe_item| *recipe_item == item)
                    .is_some()
            }) {
                None => {
                    // Should check if the output recipe requires some action to produce
                    // if not return true
                    // if yes, check if it can do this action (for example, mine asteroids or collect gas from nebula, or some other natural occurring resource)
                    todo!("has only and output recipe")
                }
                Some(recipe) => {
                    assert!(!recipe.input.is_empty());

                    let has_all_ingredients_on_market = recipe.input.items().all(|item| {
                        search_result
                            .average_buy_offers
                            .iter()
                            .find(|(offer_item, _)| *offer_item == item)
                            .is_some()
                    });

                    has_all_ingredients_on_market
                }
            }
        })
        .collect()
}

enum ChooseItemToProduceResult {
    Craft {
        item: ItemId,
        recipe: ItemRecipeHash,
    },
    ProduceFromEnvironment {
        // TODO for example: mine, collect gas, collect solar energy, etc.
    },
    NotFound,
}

fn choose_item_to_produce(
    search_result: &FindBestOffersForItemsResult,
    output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
    recipes_to_consider: &BTreeSet<ItemRecipe>,
) -> ChooseItemToProduceResult {
    // println!("search_result.max_profit_sell_offers: {:#?}", search_result.max_profit_sell_offers.iter().map(|(item, offer)|(item.clone(), offer.offer.price_per_unit.clone())).collect::<BTreeMap<ItemId, Money>>());
    // println!("search_result.average_sell_offers: {:#?}", search_result.average_sell_offers);

    let items_in_demand = calc_items_in_demand(&search_result, &output_recipes_to_consider);

    if !items_in_demand.is_empty() {
        // items_in_demand.iter().for_each(|(item, _)| {
        //     println!("items_in_demand: {:?}", item);
        // });

        let items_in_demand_that_no_one_produces =
            calc_items_in_demand_that_no_one_produces(&search_result, items_in_demand);
        if !items_in_demand_that_no_one_produces.is_empty() {
            // for (i, _) in &items_in_demand_that_no_one_produces {
            //     println!("items_in_demand_that_no_one_produces: {:?}", i);
            // }

            let items_in_demand_that_no_one_produces_and_ingredients_are_on_market =
                calc_items_in_demand_that_no_one_produces_and_ingredients_are_on_market(
                    &search_result,
                    items_in_demand_that_no_one_produces.clone(),
                    recipes_to_consider,
                );
            if !items_in_demand_that_no_one_produces_and_ingredients_are_on_market.is_empty() {
                // items_in_demand_that_no_one_produces_and_ingredients_are_on_market.iter().for_each(|(item, _)| {
                //     println!("items_in_demand_that_no_one_produces_and_ingredients_are_on_market: {:?}", item);
                // });

                todo!(
                    "Pick first item in list items_in_demand_that_no_one_produces_and_ingredients_are_on_market"
                )
            } else {
                let (item, _) = items_in_demand_that_no_one_produces
                    .first_key_value()
                    .unwrap();

                let item_recipe = recipes_to_consider
                    .iter()
                    .find(|recipe| recipe.output.contains(item))
                    .unwrap();

                // TODO: if `item_recipe` not found try:
                // output_recipes_to_consider.iter().find(|recipe| recipe.contains(item))

                ChooseItemToProduceResult::Craft {
                    item: item.clone(),
                    recipe: item_recipe.default_hash(),
                }
            }
        } else {
            todo!(
                "Find item with best profit setting buy price as cheap as possible while sustaining margin"
            )
        }
    } else {
        todo!("Has no items in demand. wait until some items are needed")
    }
}
