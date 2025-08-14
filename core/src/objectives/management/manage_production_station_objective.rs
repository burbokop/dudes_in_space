use crate::objectives::crafting::{
    CraftItemsObjective, CraftModulesObjective, CraftModulesObjectiveError,
};
use dudes_in_space_api::environment::{EnvironmentContext, FindBestOffersForItemsResult};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::recipe::ItemRecipe;
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use serde_intermediate::{Intermediate, to_intermediate};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};
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
        recipes_to_consider: Vec<ItemRecipe>,
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

        Self::CheckAllPrerequisites {
            recipes_to_consider: VecDeque::new(),
        }
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
        this_person: &PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CollectAllAvailableRecipes => {
                let vessel_item_recipes = this_vessel
                    .modules_with_capability(ModuleCapability::ItemCrafting)
                    .iter()
                    .map(|crafter| crafter.item_recipes().iter())
                    .chain(
                        this_module
                            .crafting_console()
                            .iter()
                            .map(|x| x.item_recipes().iter()),
                    )
                    .flatten()
                    .collect();
                
                let potential_vessel_item_recipes = this_vessel
                    .modules_with_capability(ModuleCapability::ModuleCrafting)
                    .iter()
                    .map(|crafter| crafter.assembly_recipes().iter().map(|recipe|recipe.output_description().item_recipes().iter()))
                    .chain(
                        this_module
                            .crafting_console()
                            .iter()
                            .map(|x| x.assembly_recipes().iter().map(|recipe|recipe.output_description().item_recipes().iter())),
                    )
                    .flatten()
                    .collect();

                let all_vessel_item_recipes = vessel_item_recipes + potential_vessel_item_recipes;
                

                while let Some(recipe) = recipes_to_consider.pop_front() {
                    if recipe.output.len() == 1 {
                        let (item, _) = recipe.output.first().unwrap();

                        logger.info("ManageProductionStationObjective::FindBestOffersForItem");
                        *self = Self::FindBestOffersForItem {
                            future: FindBestOffersForItem { item: item.clone() }
                                .push(environment_context.request_storage_mut()),
                            recipes_to_consider: std::mem::take(recipes_to_consider),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }

                struct ItemCrafter {
                    item_recipes: Vec<ItemRecipe>,
                }

                let item_crafter_modules =
                    this_vessel.modules_with_capability(ModuleCapability::ItemCrafting);

                let item_crafter_modules: Vec<_> = item_crafter_modules
                    .iter()
                    .map(|x| ItemCrafter {
                        item_recipes: x.item_recipes().iter().cloned().collect(),
                    })
                    .chain(
                        this_module
                            .capabilities()
                            .contains(&ModuleCapability::ItemCrafting)
                            .then_some(this_module.crafting_console())
                            .flatten()
                            .map(|console| ItemCrafter {
                                item_recipes: console.item_recipes().iter().cloned().collect(),
                            })
                            .into_iter(),
                    )
                    .collect();

                if item_crafter_modules.is_empty() {
                    logger.info("Crafting fabricator...");
                    *self = Self::CraftFabricator {
                        craft_objective: CraftModulesObjective::new(
                            this_person.clone(),
                            vec![ModuleCapability::ItemCrafting],
                            vec![],
                            true,
                            logger,
                        ),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                logger.info("Considering recipes:");
                for c in &item_crafter_modules {
                    logger.info(
                        format!(
                            "\t{:?}",
                            c.item_recipes
                                .iter()
                                .map(|r| r
                                    .output
                                    .clone()
                                    .into_iter()
                                    .map(|x| x.0.clone())
                                    .collect::<Vec<_>>())
                                .collect::<Vec<_>>()
                        )
                        .as_str(),
                    );
                }

                *recipes_to_consider = item_crafter_modules
                    .into_iter()
                    .map(|x| x.item_recipes)
                    .flatten()
                    .collect();
                Ok(ObjectiveStatus::InProgress)
            }
            Self::FindBestOffersForItem {
                future,
                recipes_to_consider,
            } => match future.take() {
                Ok(x) => match x.max_profit_buy_offer {
                    None => todo!(),
                    Some(offer) => todo!(),
                },
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::ExecuteProduction {
                second_attempt,
                craft_objective,
            } => {
                todo!()
            }
            Self::CraftFabricator { craft_objective } => match craft_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Done) => {
                    logger.info("Checking all prerequisites to managing production station...");
                    *self = Self::CheckAllPrerequisites {
                        recipes_to_consider: VecDeque::new(),
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
        person_id: &PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if passions.contains(&Passion::Management) || passions.contains(&Passion::Ruling) {
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
}

impl Display for ManageProductionStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ManageProductionStationObjectiveError {}
