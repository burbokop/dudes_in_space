use crate::objectives::common::MoveToModuleObjective;
use crate::objectives::crafting::{
    CraftModulesObjectiveError, CraftVesselFromScratchObjective, RequireModulesObjective,
};
use crate::objectives::trade::{PlaceBuyCustomVesselOfferObjective, PlaceSellOffersObjective};
use burbomath::math::{NonNeg, map_into_range};
use burbomath::range::RangeInclusive;
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult, RequestStorage,
};
use dudes_in_space_api::finance::{Currency, Money};
use dudes_in_space_api::item::{ItemCount, ItemId, ItemVolume};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonLogger, ThisPerson,
    ThisVessel, tie,
};
use dudes_in_space_api::trade::OrderId;
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
    },
    RequireModules {
        objective: RequireModulesObjective,
        prices_on_market: BTreeMap<ItemId, Money>,
        input_offers: BTreeMap<ItemId, (RangeInclusive<ItemCount>, Money)>,
    },
    PlaceBuyCustomVesselOffer {
        objective: PlaceBuyCustomVesselOfferObjective,
        input_offers: BTreeMap<ItemId, (RangeInclusive<ItemCount>, Money)>,
    },
    PlaceSellOffers {
        terminal_to_observe: ModuleId,
        objective: PlaceSellOffersObjective,
    },
    CheckOrders {
        move_objective: MoveToModuleObjective,
    },
    ProcessOrder {
        order: OrderId,
        craft_objective: CraftVesselFromScratchObjective,
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
    type Result = ();
    type Error = ManageDockyardStationObjectiveError;

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
                logger.info("ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe");

                *self = Self::FindBestOffersAndDecideBestRecipe {
                    future: find_best_offers_for_input_items(
                        tie(this_module, this_vessel),
                        environment_context.request_storage_mut(),
                    ),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::FindBestOffersAndDecideBestRecipe { future } => match future.take() {
                Ok(search_result) => {
                    let assembly_recipes: BTreeSet<_> = iter::chain(
                        tie(this_module, this_vessel).assembly_recipes().into_iter(),
                        tie(this_module, this_vessel)
                            .potential_assembly_recipes()
                            .into_iter(),
                    )
                    .collect();

                    let mut min_counts: BTreeMap<ItemId, ItemCount> = BTreeMap::new();
                    assembly_recipes
                        .iter()
                        .map(|a| a.input())
                        .flatten()
                        .for_each(|(item, count)| {
                            let c = min_counts.entry(item.clone()).or_default();
                            *c = ItemCount::max(*c, *count);
                        });

                    let sum_volume: ItemVolume = min_counts
                        .clone()
                        .into_iter()
                        .map(|(item, count)| {
                            environment_context
                                .item_vault()
                                .get_ref(item)
                                .unwrap()
                                .volume
                                * count
                        })
                        .sum();

                    // I use minimum free space storage because I don't know which one ends up to be used
                    // TODO pick specific storage and remember it in objective, free it from junk and and dedicate only for assembling

                    let this_vessel = tie(this_module, this_vessel);

                    let min_free_space_storage = this_vessel
                        .storages()
                        .into_iter()
                        .min_by(|a, b| a.free_space().cmp(&b.free_space()))
                        .unwrap();

                    let capacity_dedicated_for_this_objective = min_free_space_storage.free_space();

                    let mut input_offers: BTreeMap<String, (RangeInclusive<ItemCount>, Money)> =
                        Default::default();

                    for (item, count) in min_counts {
                        let item = environment_context.item_vault().get_ref(item).unwrap();

                        let item_volume = item.volume * count;
                        let portion: f32 = item_volume / sum_volume;
                        let cap_for_item: ItemVolume =
                            capacity_dedicated_for_this_objective * portion;
                        let capacity_for_item = (cap_for_item / item.volume) as ItemCount;

                        let this_person_wallet_id = this_person.finance.wallet().id().clone();
                        let target_currency: Currency =
                            this_person.preferred_currency_or_create_default(environment_context);

                        let cheapest_buy_offer = search_result
                            .max_profit_buy_offers
                            .get(&item.id)
                            .map(|offer| {
                                offer
                                    .offer
                                    .price_per_unit
                                    .convert_to_currency(
                                        environment_context.bank_registry(),
                                        target_currency.clone(),
                                    )
                                    .unwrap()
                                    .amount
                                    .unwrap()
                            })
                            .unwrap_or(1);

                        let average_buy_offer = search_result
                            .average_buy_offers
                            .get(&item.id)
                            .map(|price| {
                                price
                                    .convert_to_currency(
                                        environment_context.bank_registry(),
                                        target_currency.clone(),
                                    )
                                    .unwrap()
                                    .amount
                                    .unwrap()
                            })
                            .unwrap_or(1);

                        let price_for_item = if count < capacity_for_item / 2 {
                            map_into_range(
                                count as i64,
                                0..capacity_for_item as i64 / 2,
                                (average_buy_offer * 2)..average_buy_offer,
                            )
                        } else {
                            map_into_range(
                                count as i64,
                                (capacity_for_item as i64 / 2)..capacity_for_item as i64,
                                average_buy_offer..cheapest_buy_offer,
                            )
                        };

                        let occupied_space = min_free_space_storage.count(item.id.clone());
                        // TODO: if occupied_space == capacity_for_item then no need for order
                        assert!(occupied_space < capacity_for_item);

                        let count_range = (1..=(capacity_for_item - occupied_space)).into();

                        input_offers.insert(
                            item.id.clone(),
                            (
                                count_range,
                                Money {
                                    currency: target_currency,
                                    amount: NonNeg::new(price_for_item).unwrap(),
                                },
                            ),
                        );
                    }

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
                        prices_on_market: search_result
                            .max_profit_buy_offers
                            .into_iter()
                            .map(|(item, offer)| (item, offer.offer.price_per_unit))
                            .collect(),
                        input_offers,
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::RequireModules {
                objective,
                prices_on_market,
                input_offers,
            } => {
                match objective.pursue(
                    this_person,
                    this_module,
                    this_vessel,
                    environment_context,
                    logger,
                ) {
                    Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                    Ok(ObjectiveStatus::Passive) => todo!(),
                    Ok(ObjectiveStatus::Done(_)) => {
                        *self = Self::PlaceBuyCustomVesselOffer {
                            objective: PlaceBuyCustomVesselOfferObjective::new(std::mem::take(
                                prices_on_market,
                            )),
                            input_offers: std::mem::take(input_offers),
                        };
                        Ok(ObjectiveStatus::InProgress)
                    }
                    Err(err) => Err(Self::Error::CanNotCraftRequiredModules(err)),
                }
            }
            Self::PlaceBuyCustomVesselOffer {
                objective,
                input_offers,
            } => match objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Passive) => todo!(),
                Ok(ObjectiveStatus::Done(result)) => {
                    *self = Self::PlaceSellOffers {
                        objective: PlaceSellOffersObjective::new(std::mem::take(input_offers)),
                        terminal_to_observe: result.target_terminal,
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(err) => todo!("{} {:?}", this_person.id, err),
            },

            Self::PlaceSellOffers {
                objective,
                terminal_to_observe,
            } => match objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Passive) => todo!(),
                Ok(ObjectiveStatus::Done(_)) => {
                    *self = Self::CheckOrders {
                        move_objective: MoveToModuleObjective::new(*terminal_to_observe),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(err) => todo!(),
            },
            Self::CheckOrders { move_objective } => {
                match move_objective.pursue(
                    this_person,
                    this_module,
                    this_vessel,
                    environment_context,
                    logger,
                ) {
                    Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                    Ok(ObjectiveStatus::Passive) => todo!(),
                    Ok(ObjectiveStatus::Done(_)) => {
                        assert!(
                            this_module
                                .capabilities()
                                .contains(&ModuleCapability::VesselSellingTerminal)
                        );

                        let console = this_module.trading_admin_console_mut().unwrap();

                        if let Some(current_order) = console.buy_vessel_orders().first() {
                            // - find recipes for caps
                            // - make a list of all input ingredients
                            // - place sell offers for all input ingredients

                            todo!()
                        }

                        if let Some(current_order) = console.buy_custom_vessel_orders().first() {
                            let caps = current_order.primary_capabilities();

                            // current_order.

                            // - find recipes for caps
                            // - make a list of all input ingredients
                            // - place sell offers for all input ingredients

                            let x = this_person.notes.purchased_items_max_prices().stabilized();
                            if x.is_empty() {
                                println!(
                                    "Wait until this_person.notes.purchased_items_max_prices fill up"
                                )
                            } else {
                                todo!()
                            }
                        }

                        Ok(ObjectiveStatus::InProgress)
                    }
                    Err(_) => todo!(),
                }
            }
            Self::ProcessOrder {
                order,
                craft_objective,
            } => match craft_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Passive) => todo!(),
                Ok(ObjectiveStatus::Done(result)) => {
                    todo!("BuildVesselObjective resulted with: {:?}", result)
                }
                Err(_) => todo!(),
            },
        }
    }
}

#[derive(Debug)]
enum ManageDockyardStationObjectiveError {
    CanNotCraftRequiredModules(CraftModulesObjectiveError),
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
        person: &ThisPerson,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if (person.passions.contains(&Passion::Management)
            || person.passions.contains(&Passion::Ruling))
            && person.passions.contains(&Passion::Vessels)
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

impl Display for ManageDockyardStationObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ManageDockyardStationObjective::CollectAllAvailableRecipes => {
                write!(f, "CollectAllAvailableRecipes")
            }
            ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe { .. } => {
                write!(f, "FindBestOffersAndDecideBestRecipe")
            }
            ManageDockyardStationObjective::RequireModules { .. } => write!(f, "RequireModules"),
            ManageDockyardStationObjective::CheckOrders { .. } => write!(f, "CheckOrders"),
            ManageDockyardStationObjective::PlaceBuyCustomVesselOffer { .. } => {
                write!(f, "PlaceBuyCustomVesselOffer")
            }
            ManageDockyardStationObjective::PlaceSellOffers { .. } => write!(f, "PlaceSellOffers"),
            ManageDockyardStationObjective::ProcessOrder { .. } => todo!(),
        }
    }
}

fn find_best_offers_for_input_items(
    this_vessel: ThisVessel,
    request_storage: &mut RequestStorage,
) -> ReqFuture<FindBestOffersForItemsResult> {
    let assembly_recipes: Vec<_> = iter::chain(
        this_vessel.assembly_recipes().into_iter(),
        this_vessel.potential_assembly_recipes().into_iter(),
    )
    .collect();

    let items: BTreeSet<_> = assembly_recipes
        .iter()
        .map(|x| x.input().items())
        .flatten()
        .cloned()
        .collect();

    FindBestOffersForItems { items }.push(request_storage)
}
