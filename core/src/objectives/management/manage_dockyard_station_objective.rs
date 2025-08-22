use crate::objectives::crafting::RequireModulesObjective;
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult,
};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::recipe::{InputItemRecipe, OutputItemRecipe};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
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
    - Find list of modules you can craft
    - Place available capabilities in vessel selling terminal
    - Check orders || Craft your own designs

        Check orders
            - Sit at terminal and check orders
            - When order found: produce
            - Give vessel to a customer
            - Save design if u like it

        Craft your own designs
            - Load designs
            - Craft design
            - Place offer in terminal
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
    RequireModules {
        objective: RequireModulesObjective,
    },
    MoveToTerminal {
        dst: ModuleId,
    },
    PlaceOffers,
    CheckOrders,
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
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CollectAllAvailableRecipes => {
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
            Self::MoveToTerminal { dst } => {
                if *dst == this_module.id() {
                    logger.info("Placing capabilities in vessel selling terminal...");
                    *self = Self::PlaceOffers;
                    Ok(ObjectiveStatus::InProgress)
                } else {
                    logger.info("Entering vessel selling terminal module...");
                    match this_vessel.move_person_to_module(
                        environment_context.subordination_table(),
                        *this_person.id,
                        *dst,
                    ) {
                        Ok(_) => Ok(ObjectiveStatus::InProgress),
                        Err(MoveToModuleError::ModuleNotFound) => {
                            Err(ManageDockyardStationObjectiveError::VesselSellingTerminalMissing)
                        }
                        Err(MoveToModuleError::PermissionDenied) => {
                            Err(ManageDockyardStationObjectiveError::PermissionsDenied)
                        }
                        Err(MoveToModuleError::NotEnoughSpace) => {
                            logger.info(
                                "Not enough space in crafting module. Searching another one...",
                            );
                            todo!()
                        }
                    }
                }
            }
            Self::PlaceOffers => {
                let assembly_recipes: Vec<_> = iter::chain(
                    person::utils::this_vessel_assembly_recipes(this_module, this_vessel)
                        .into_iter(),
                    person::utils::this_vessel_potential_assembly_recipes(this_module, this_vessel)
                        .into_iter(),
                )
                .collect();

                let console = this_module.trading_admin_console_mut().unwrap();

                console.place_buy_custom_vessel_offer(
                    assembly_recipes
                        .iter()
                        .map(|x| x.output_description().capabilities().iter())
                        .flatten()
                        .cloned()
                        .collect(),
                    assembly_recipes
                        .iter()
                        .map(|x| x.output_description().primary_capabilities().iter())
                        .flatten()
                        .cloned()
                        .collect(),
                );

                *self = Self::CheckOrders;
                Ok(ObjectiveStatus::InProgress)
            }
            Self::FindBestOffersAndDecideBestRecipe {
                future,
                input_recipes_to_consider,
                output_recipes_to_consider,
            } => match future.take() {
                Ok(search_result) => {
                    let assembly_recipes: BTreeSet<_> = iter::chain(
                        person::utils::this_vessel_assembly_recipes(this_module, this_vessel)
                            .into_iter(),
                        person::utils::this_vessel_potential_assembly_recipes(
                            this_module,
                            this_vessel,
                        )
                        .into_iter(),
                    )
                    .collect();

                    println!(
                        "ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe: {:#?}",
                        (search_result, assembly_recipes)
                    );

                    *self = Self::RequireModules {
                        objective: RequireModulesObjective::new(
                            BTreeSet::from([
                                ModuleCapability::VesselSellingTerminal,
                                ModuleCapability::TradingTerminal,
                                ModuleCapability::Dockyard,
                                ModuleCapability::ModuleCrafting,
                                ModuleCapability::ModuleStorage,
                            ]),
                            BTreeSet::new(),
                            logger,
                        ),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            ManageDockyardStationObjective::RequireModules { objective } => {
                match objective.pursue(
                    this_person,
                    this_module,
                    this_vessel,
                    environment_context,
                    logger,
                ) {
                    Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                    Ok(ObjectiveStatus::Done) => {
                        if this_module
                            .capabilities()
                            .contains(&ModuleCapability::VesselSellingTerminal)
                        {
                            *self = Self::PlaceOffers;
                            return Ok(ObjectiveStatus::InProgress);
                        }

                        let terminals = this_vessel
                            .modules_with_capability(ModuleCapability::VesselSellingTerminal);

                        if terminals.len() == 0 {
                            return Err(
                                ManageDockyardStationObjectiveError::VesselSellingTerminalMissing,
                            );
                        }

                        *self = Self::MoveToTerminal {
                            dst: terminals.first().unwrap().id(),
                        };

                        Ok(ObjectiveStatus::InProgress)
                    }
                    Err(err) => {
                        todo!()
                    }
                }
            }
            ManageDockyardStationObjective::CheckOrders => {
                let console = this_module.trading_admin_console_mut().unwrap();

                let orders = console.buy_vessel_orders();
                if orders.is_empty() {
                    return Ok(ObjectiveStatus::InProgress);
                }

                let current_order = &orders[0];
                let caps = current_order.primary_caps();

                // - find recipes for caps
                // - make a list of all input ingredients
                // - place sell offers for all input ingredients

                todo!()
            }
        }
    }
}

#[derive(Debug)]
enum ManageDockyardStationObjectiveError {
    VesselSellingTerminalMissing,
    PermissionsDenied,
}

impl Display for ManageDockyardStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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
