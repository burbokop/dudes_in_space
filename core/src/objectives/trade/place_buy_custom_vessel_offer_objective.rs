use burbomath::math::NonNeg;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{Money, MoneyAmount};
use dudes_in_space_api::item::ItemId;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person::{tie, Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe};
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter;

#[derive(Debug, Serialize, Deserialize)]
// #[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "place_buy_custom_vessel_offer_objective_stage")]
// #[deserialize_seed_xxx(seed = crate::objectives::management::place_buy_custom_vessel_offer_objective::PlaceBuyCustomVesselOfferObjectiveSeed::<'context>)]
pub(crate) enum PlaceBuyCustomVesselOfferObjective {
    FindTerminal {
        input_prices: BTreeMap<ItemId, Money>,
    },
    MoveToTerminal {
        dst: ModuleId,
        input_prices: BTreeMap<ItemId, Money>,
    },
    PlaceOffer {
        target_terminal: ModuleId,
        input_prices: BTreeMap<ItemId, Money>,
    },
    Done {
        target_terminal: ModuleId,
    },
}

struct PlaceBuyCustomVesselOfferObjectiveSeed {}

impl PlaceBuyCustomVesselOfferObjective {
    pub fn new(input_prices: BTreeMap<ItemId, Money>) -> Self {
        Self::FindTerminal { input_prices }
    }
}

pub(crate) struct CraftVesselFromScratchObjectiveResult {
    pub(crate) target_terminal: ModuleId,
}

impl Objective for PlaceBuyCustomVesselOfferObjective {
    type Result = CraftVesselFromScratchObjectiveResult;
    type Error = PlaceBuyCustomVesselOfferObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::FindTerminal { input_prices } => {
                if this_module
                    .capabilities()
                    .contains(&ModuleCapability::VesselSellingTerminal)
                {
                    *self = Self::PlaceOffer {
                        target_terminal: this_module.id(),
                        input_prices: std::mem::take(input_prices),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                let terminals =
                    this_vessel.modules_with_capability(ModuleCapability::VesselSellingTerminal);

                if terminals.len() == 0 {
                    return Err(Self::Error::VesselSellingTerminalMissing);
                }

                *self = Self::MoveToTerminal {
                    dst: terminals.first().unwrap().id(),
                    input_prices: std::mem::take(input_prices),
                };

                Ok(ObjectiveStatus::InProgress)
            }
            Self::MoveToTerminal { dst, input_prices } => {
                if *dst == this_module.id() {
                    logger.info("Placing capabilities in vessel selling terminal...");
                    *self = Self::PlaceOffer {
                        target_terminal: *dst,
                        input_prices: std::mem::take(input_prices),
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
                        Err(MoveToModuleError::ModuleNotFound) => Err(
                            PlaceBuyCustomVesselOfferObjectiveError::VesselSellingTerminalMissing,
                        ),
                        Err(MoveToModuleError::PermissionDenied) => {
                            Err(PlaceBuyCustomVesselOfferObjectiveError::PermissionsDenied)
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
            Self::PlaceOffer {
                target_terminal,
                input_prices,
            } => {
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
                        currency: person.preferred_currency_or_create_default(environment_context),
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
                        &input_prices,
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
                let target_terminal = *target_terminal;
                *self = Self::Done { target_terminal };
                Ok(ObjectiveStatus::Done(Self::Result { target_terminal }))
            }
            Self::Done { target_terminal } => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum PlaceBuyCustomVesselOfferObjectiveError {
    VesselSellingTerminalMissing,
    PermissionsDenied,
}

impl Display for PlaceBuyCustomVesselOfferObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PlaceBuyCustomVesselOfferObjectiveError {}
