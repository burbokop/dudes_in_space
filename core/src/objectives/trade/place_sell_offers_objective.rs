use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::Money;
use dudes_in_space_api::item::{ItemCount, ItemId};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::utils::range::RangeInclusive;
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
// #[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "place_buy_custom_vessel_offer_objective_stage")]
// #[deserialize_seed_xxx(seed = crate::objectives::management::place_buy_custom_vessel_offer_objective::PlaceBuyCustomVesselOfferObjectiveSeed::<'context>)]
pub(crate) enum PlaceSellOffersObjective {
    FindTerminal {
        offers: BTreeMap<ItemId, (RangeInclusive<ItemCount>, Money)>,
    },
    MoveToTerminal {
        dst: ModuleId,
        offers: BTreeMap<ItemId, (RangeInclusive<ItemCount>, Money)>,
    },
    PlaceOffers {
        offers: BTreeMap<ItemId, (RangeInclusive<ItemCount>, Money)>,
    },
    Done,
}

struct PlaceSellOfferObjectiveSeed {}

impl PlaceSellOffersObjective {
    pub fn new(offers: BTreeMap<ItemId, (RangeInclusive<ItemCount>, Money)>) -> Self {
        Self::FindTerminal { offers }
    }
}

impl Objective for PlaceSellOffersObjective {
    type Result = ();
    type Error = PlaceSellOfferObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::FindTerminal { offers } => {
                if this_module
                    .capabilities()
                    .contains(&ModuleCapability::TradingTerminal)
                {
                    *self = Self::PlaceOffers {
                        offers: std::mem::take(offers),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                let terminals =
                    this_vessel.modules_with_capability(ModuleCapability::TradingTerminal);

                if terminals.len() == 0 {
                    return Err(Self::Error::TradingTerminalMissing);
                }

                *self = Self::MoveToTerminal {
                    dst: terminals.first().unwrap().id(),
                    offers: std::mem::take(offers),
                };

                Ok(ObjectiveStatus::InProgress)
            }
            Self::MoveToTerminal { dst, offers } => {
                if *dst == this_module.id() {
                    logger.info("Placing offers in trading terminal...");
                    *self = Self::PlaceOffers {
                        offers: std::mem::take(offers),
                    };
                    Ok(ObjectiveStatus::InProgress)
                } else {
                    logger.info("Entering trading terminal module...");
                    match this_vessel.move_person_to_module(
                        environment_context.subordination_table(),
                        *this_person.id,
                        *dst,
                    ) {
                        Ok(_) => Ok(ObjectiveStatus::InProgress),
                        Err(MoveToModuleError::ModuleNotFound) => {
                            Err(Self::Error::TradingTerminalMissing)
                        }
                        Err(MoveToModuleError::PermissionDenied) => {
                            Err(Self::Error::PermissionsDenied)
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
            Self::PlaceOffers { offers } => {
                let console = this_module.trading_admin_console_mut().unwrap();

                for (item, (count_range, price_per_item)) in offers.iter() {
                    console.place_sell_offer(
                        item.clone(),
                        count_range.clone(),
                        price_per_item.clone(),
                    );
                }

                *self = Self::Done;
                Ok(ObjectiveStatus::Done(()))
            }
            Self::Done => Ok(ObjectiveStatus::Done(())),
        }
    }
}

#[derive(Debug)]
pub(crate) enum PlaceSellOfferObjectiveError {
    TradingTerminalMissing,
    PermissionsDenied,
}

impl Display for PlaceSellOfferObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PlaceSellOfferObjectiveError {}
