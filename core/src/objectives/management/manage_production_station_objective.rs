use crate::objectives::crafting::{
    CraftItemsObjective, CraftModulesObjective, CraftModulesObjectiveError,
};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult,
};
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::recipe::{InputItemRecipe, ItemRecipe, OutputItemRecipe};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use serde_intermediate::{Intermediate, to_intermediate};
use std::collections::BTreeSet;
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
        craft_objective: CraftItemsObjective,
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
    type Error = ManageProductionStationObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CollectAllAvailableRecipes => {
                let item_recipes: BTreeSet<_> = iter::chain(
                    person::utils::this_vessel_item_recipes(this_module, this_vessel).into_iter(),
                    person::utils::this_vessel_potential_item_recipes(this_module, this_vessel)
                        .into_iter(),
                )
                .collect();

                let input_item_recipes: BTreeSet<_> = iter::chain(
                    person::utils::this_vessel_input_item_recipes(this_module, this_vessel)
                        .into_iter(),
                    person::utils::this_vessel_potential_input_item_recipes(
                        this_module,
                        this_vessel,
                    )
                    .into_iter(),
                )
                .collect();

                let output_item_recipes: BTreeSet<_> = iter::chain(
                    person::utils::this_vessel_output_item_recipes(this_module, this_vessel)
                        .into_iter(),
                    person::utils::this_vessel_potential_output_item_recipes(
                        this_module,
                        this_vessel,
                    )
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
                Ok(search_result) => {
                    if search_result.max_profit_sell_offers.is_empty() {
                        Err(Self::Error::NoSellOffersFound)
                    } else {
                        logger.info(format!(
                            "Trade scan summary: {:#?}",
                            (
                                recipes_to_consider,
                                input_recipes_to_consider,
                                output_recipes_to_consider,
                                &search_result
                            )
                        ));

                        todo!()
                    }
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::ExecuteProduction { craft_objective } => {
                todo!()
            }
            Self::AssembleCrafter { craft_objective } => match craft_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done) => {
                    logger.info("Checking all prerequisites to managing production station...");
                    *self = Self::ExecuteProduction {
                        craft_objective: CraftItemsObjective::new(
                            todo!(),
                            #[allow(unreachable_code)]
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
        person: &PersonInfo,
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
