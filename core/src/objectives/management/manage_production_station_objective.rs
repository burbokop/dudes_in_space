use std::collections::VecDeque;
use crate::objectives::crafting::{CraftItemsObjective, CraftModulesObjective};
use dudes_in_space_api::environment::{EnvironmentContext, FindBestOffersForItem, FindBestOffersForItemResult};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use serde::{ Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use dudes_in_space_api::recipe::ItemRecipe;
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dyn_serde_macro::DeserializeSeedXXX;

static TYPE_ID: &str = "ManageProductionStationObjective";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "manage_production_station_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::management::manage_production_station_objective::ManageProductionStationObjectiveSeed::<'context>)]
pub(crate) enum ManageProductionStationObjective {
    CheckAllPrerequisites {
        recipes_to_consider: VecDeque<ItemRecipe>,
    },
    CraftFabricator {
        craft_objective: CraftModulesObjective,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    FindBestOffersForItem {
        future: ReqFuture<FindBestOffersForItemResult>,
        recipes_to_consider: VecDeque<ItemRecipe>,
    },
    ExecuteProduction {
        second_attempt: bool,
        craft_objective: CraftItemsObjective,
    },
}

impl ManageProductionStationObjective {
    pub(crate) fn new() -> Self {
        Self::CheckAllPrerequisites {
            recipes_to_consider: VecDeque::new(),
        }
    }
}

struct ManageProductionStationObjectiveSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindBestOffersForItemResult>,
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
            Self::CheckAllPrerequisites {recipes_to_consider} => {

                while let Some(recipe) = recipes_to_consider.pop_front() {
                    if recipe.output.len() == 1 {
                        let (item, _) = recipe.output.first().unwrap()
                            ;

                        *self = Self::FindBestOffersForItem {
                            future: FindBestOffersForItem {
                                item: item.clone(),
                            }
                                .push(environment_context.request_storage_mut()),
                            recipes_to_consider: std::mem::take(recipes_to_consider),
                        };
 return                       Ok(ObjectiveStatus::InProgress)
                    }
                }


                struct ItemCrafter {
                    item_recipes: Vec<ItemRecipe>
                }

                let item_crafter_modules = this_vessel.modules_with_capability(ModuleCapability::ItemCrafting);

                let item_crafter_modules: Vec<_> = item_crafter_modules
                    .iter()
                    .map(|x| ItemCrafter {
                        item_recipes: x.item_recipes().iter().cloned().collect(),
                    })
                    .chain(
                        this_module
                            .capabilities()
                            .contains(&ModuleCapability::ItemCrafting)
                            .then_some(
                                this_module.crafting_console()
                                ).flatten().map(|console|ItemCrafter {
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

               *recipes_to_consider = item_crafter_modules.into_iter().map(|x|x.item_recipes).flatten().collect();
                Ok(ObjectiveStatus::InProgress)
            }
            Self::FindBestOffersForItem { future, recipes_to_consider } => {
                match future.take() {
                    Ok(x) => todo!(),
                    Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                    Err(ReqTakeError::AlreadyTaken) => unreachable!(),
                }
            },
            Self::ExecuteProduction { second_attempt, craft_objective } => todo!(),
            Self::CraftFabricator { craft_objective } => todo!(),
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
            Some(Box::new(ManageProductionStationObjective::new()))
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
    req_context: Rc<ReqContext>
}

impl ManageProductionStationObjectiveDynSeed {
    pub fn new(req_context: Rc<ReqContext>) -> Self {
        Self {
            req_context
        }
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
        let obj: ManageProductionStationObjective =
            from_intermediate_seed(ManageProductionStationObjectiveSeed::new(&self.req_context), &intermediate)
                .map_err(Box::new)?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug)]
pub(crate) enum ManageProductionStationObjectiveError {}

impl Display for ManageProductionStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ManageProductionStationObjectiveError {}
