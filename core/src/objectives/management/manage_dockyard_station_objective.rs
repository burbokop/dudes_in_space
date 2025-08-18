use dudes_in_space_api::environment::{EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult};
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::recipe::{
    InputItemRecipe, 
                                  OutputItemRecipe};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed};
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
use dudes_in_space_api::person;
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

static TYPE_ID: &str = "ManageDockyardStationObjective";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "manage_dockyard_station_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::management::manage_dockyard_station_objective::ManageDockyardStationObjectiveSeed::<'context>)]
enum ManageDockyardStationObjective {
    CollectAllAvailableRecipes,
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    FindBestOffersAndDecideBestRecipe {
        future: ReqFuture<FindBestOffersForItemsResult>,
        input_recipes_to_consider: BTreeSet<InputItemRecipe>,
        output_recipes_to_consider: BTreeSet<OutputItemRecipe>,
    },
}

struct ManageDockyardStationObjectiveSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindBestOffersForItemsResult>,
}

impl<'context> ManageDockyardStationObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            req_future_seed: ReqFutureSeed::new(context),
        }
    }
}

impl ManageDockyardStationObjective {
    pub(crate) fn new(logger: &mut PersonLogger) -> Self {
        Self::CollectAllAvailableRecipes
    }
}

impl Objective for ManageDockyardStationObjective {
    type Error = ManageDockyardStationObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            ManageDockyardStationObjective::CollectAllAvailableRecipes =>


                {
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

                    logger.info("ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe");
                    *self = Self::FindBestOffersAndDecideBestRecipe {
                        future: FindBestOffersForItems { items }
                            .push(environment_context.request_storage_mut()),
                        input_recipes_to_consider: input_item_recipes,
                        output_recipes_to_consider: output_item_recipes,
                    };
                    Ok(ObjectiveStatus::InProgress)
                    
                    
                }

            
            ,
            ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe {
                future,
                input_recipes_to_consider,
                output_recipes_to_consider,
            } => {

                let item_recipes: BTreeSet<_> = iter::chain(
                    person::utils::this_vessel_assembly_recipes(this_module, this_vessel).into_iter(),
                    person::utils::this_vessel_potential_assembly_recipes(this_module, this_vessel)
                        .into_iter(),
                )
                    .collect();
                
                todo!()
            },
        }
    }
}

#[derive(Debug)]
enum ManageDockyardStationObjectiveError {}

impl Display for ManageDockyardStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ManageDockyardStationObjectiveError {}

pub(crate) struct ManageDockyardStationObjectiveDecider;

impl ObjectiveDecider for ManageDockyardStationObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if person.passions.contains(&Passion::Management)
            || person.passions.contains(&Passion::Ruling)
        {
            logger.info("Manage dockyard station objective decided.");
            Some(Box::new(ManageDockyardStationObjective::new(logger)))
        } else {
            None
        }
    }
}

pub(crate) struct ManageDockyardStationObjectiveDynSeed {
    req_context: Rc<ReqContext>,
}

impl ManageDockyardStationObjectiveDynSeed {
    pub(crate) fn new(req_context: Rc<ReqContext>) -> Self {
        Self { req_context }
    }
}

impl DynDeserializeSeed<dyn DynObjective> for ManageDockyardStationObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let r: ManageDockyardStationObjective = from_intermediate_seed(
            ManageDockyardStationObjectiveSeed::new(&self.req_context),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(r))
    }
}

impl DynSerialize for ManageDockyardStationObjective {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}
