use crate::objectives::crafting::{CraftModulesObjectiveError, RequireModulesObjective};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOffersForItems, FindBestOffersForItemsResult,
};
use dudes_in_space_api::finance::{Bank, Money, MoneyAmount};
use dudes_in_space_api::item::ItemId;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonLogger, ThisPerson,
    tie,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe};
use dudes_in_space_api::utils::math::NonNeg;
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
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
    },
    MoveToTerminal {
        dst: ModuleId,
        prices_on_market: BTreeMap<ItemId, Money>,
    },
    PlaceOffers {
        prices_on_market: BTreeMap<ItemId, Money>,
    },
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
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CollectAllAvailableRecipes => {
                let assembly_recipes: Vec<_> = iter::chain(
                    tie(this_module, this_vessel).assembly_recipes().into_iter(),
                    tie(this_module, this_vessel)
                        .potential_assembly_recipes()
                        .into_iter(),
                )
                .collect();

                let items: BTreeSet<_> = assembly_recipes
                    .iter()
                    .map(|x| x.input().items())
                    .flatten()
                    .cloned()
                    .collect();

                logger.info("ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe");
                *self = Self::FindBestOffersAndDecideBestRecipe {
                    future: FindBestOffersForItems { items }
                        .push(environment_context.request_storage_mut()),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::MoveToTerminal {
                dst,
                prices_on_market,
            } => {
                if *dst == this_module.id() {
                    logger.info("Placing capabilities in vessel selling terminal...");
                    *self = Self::PlaceOffers {
                        prices_on_market: std::mem::take(prices_on_market),
                    };
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
            Self::PlaceOffers { prices_on_market } => {
                let assembly_recipes: Vec<_> = iter::chain(
                    tie(this_module, this_vessel).assembly_recipes().into_iter(),
                    tie(this_module, this_vessel)
                        .potential_assembly_recipes()
                        .into_iter(),
                )
                .collect();

                fn default_price(
                    person: &mut ThisPerson,
                    environment_context: &EnvironmentContext,
                ) -> Money {
                    Money {
                        currency: person.finance.preferred_currency_or_create(
                            environment_context.bank_registry(),
                            Bank::new(
                                person.id.clone(),
                                environment_context.currency_generator().generate_name(
                                    &mut rng(),
                                    environment_context.bank_registry(),
                                    person,
                                ),
                            ),
                        ),
                        amount: NonNeg::new(1).unwrap(),
                    }
                }

                fn price_of_input_recipe(
                    person: &mut ThisPerson,
                    environment_context: &EnvironmentContext,
                    recipe: &InputItemRecipe,
                    prices_on_market: &BTreeMap<ItemId, Money>,
                ) -> Option<Money> {
                    let mut result = default_price(person, environment_context);
                    for (item, count) in recipe {
                        result.add_assign(
                            environment_context.bank_registry(),
                            prices_on_market.get(item)?.clone() * *count,
                        );
                    }
                    Some(result)
                }

                fn prices_of_capabilities(
                    person: &mut ThisPerson,
                    environment_context: &EnvironmentContext,
                    recipes: &[AssemblyRecipe],
                    prices_on_market: &BTreeMap<ItemId, Money>,
                ) -> (
                    BTreeMap<ModuleCapability, Money>,
                    BTreeMap<ModuleCapability, Money>,
                ) {
                    let mut capabilities: BTreeMap<ModuleCapability, Money> = Default::default();
                    let mut primary_capabilities: BTreeMap<ModuleCapability, Money> =
                        Default::default();

                    let money_sum = person
                        .finance
                        .wallet()
                        .sum(environment_context.bank_registry());

                    let insert = |caps: &mut BTreeMap<ModuleCapability, Money>,
                                  cap: ModuleCapability,
                                  money: Money| {
                        caps.entry(cap)
                            .and_modify(|m| {
                                m.max_assign(environment_context.bank_registry(), money.clone());
                            })
                            .or_insert(money);
                    };

                    for recipe in recipes {
                        for cap in recipe.output_description().capabilities() {
                            insert(
                                &mut capabilities,
                                *cap,
                                price_of_input_recipe(
                                    person,
                                    environment_context,
                                    recipe.input(),
                                    prices_on_market,
                                )
                                .or(money_sum.clone())
                                .unwrap_or_else(|| default_price(person, environment_context)),
                            );
                        }

                        for cap in recipe.output_description().primary_capabilities() {
                            insert(
                                &mut primary_capabilities,
                                *cap,
                                price_of_input_recipe(
                                    person,
                                    environment_context,
                                    recipe.input(),
                                    prices_on_market,
                                )
                                .or(money_sum.clone())
                                .unwrap_or_else(|| default_price(person, environment_context)),
                            );
                        }
                    }

                    (capabilities, primary_capabilities)
                }

                let (mut capabilities_prices, mut primary_capabilities_prices) =
                    prices_of_capabilities(
                        this_person,
                        environment_context,
                        &assembly_recipes,
                        &prices_on_market,
                    );

                let console = this_module.trading_admin_console_mut().unwrap();

                for (_, price) in &mut capabilities_prices {
                    price.amount = NonNeg::new(
                        (price.amount.unwrap() as Float * this_person.notes.margin().unwrap())
                            as MoneyAmount,
                    )
                    .unwrap();
                }

                for (_, price) in &mut primary_capabilities_prices {
                    price.amount = NonNeg::new(
                        (price.amount.unwrap() as Float * this_person.notes.margin().unwrap())
                            as MoneyAmount,
                    )
                    .unwrap();
                }

                console.place_buy_custom_vessel_offer(
                    capabilities_prices,
                    primary_capabilities_prices,
                );
                *self = Self::CheckOrders;
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

                    println!(
                        "ManageDockyardStationObjective::FindBestOffersAndDecideBestRecipe: {:#?}",
                        (&search_result, assembly_recipes)
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
                        prices_on_market: search_result
                            .max_profit_buy_offers
                            .into_iter()
                            .map(|(item, offer)| (item, offer.offer.price_per_unit))
                            .collect(),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            ManageDockyardStationObjective::RequireModules {
                objective,
                prices_on_market,
            } => {
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
                            *self = Self::PlaceOffers {
                                prices_on_market: std::mem::take(prices_on_market),
                            };
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
                            prices_on_market: std::mem::take(prices_on_market),
                        };

                        Ok(ObjectiveStatus::InProgress)
                    }
                    Err(err) => Err(Self::Error::CanNotCraftRequiredModules(err)),
                }
            }
            ManageDockyardStationObjective::CheckOrders => {
                let console = this_module.trading_admin_console_mut().unwrap();

                if let Some(current_order) = console.buy_vessel_orders().first() {
                    // - find recipes for caps
                    // - make a list of all input ingredients
                    // - place sell offers for all input ingredients

                    todo!()
                }

                if let Some(current_order) = console.buy_custom_vessel_orders().first() {
                    let caps = current_order.primary_capabilities();

                    // - find recipes for caps
                    // - make a list of all input ingredients
                    // - place sell offers for all input ingredients

                    todo!()
                }

                Ok(ObjectiveStatus::InProgress)
            }
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
            ManageDockyardStationObjective::MoveToTerminal { .. } => write!(f, "MoveToTerminal"),
            ManageDockyardStationObjective::PlaceOffers { .. } => write!(f, "PlaceOffers"),
            ManageDockyardStationObjective::CheckOrders => write!(f, "CheckOrders"),
        }
    }
}
