use crate::objectives::trade::BuyVesselObjective::CheckPrerequisites;
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestBuyVesselOffer, FindBestBuyVesselOfferResult,
    PlaceBuyCustomVesselOrderResult, RequestCreditLimitIncrease, RequestCreditLimitIncreaseResult,
};
use dudes_in_space_api::finance::WithdrawalError;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ProcessTokenExpiredError};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{tie, Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::trade::WeakBuyCustomVesselOrder;
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::VesselInternalConsole;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "BuyVesselObjective";

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "buy_vessel_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::trade::buy_vessel_objective::BuyVesselObjectiveSeed::<'context>)]
pub(crate) enum BuyVesselObjective {
    CheckPrerequisites {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.find_offers_future_seed)])]
    FindOffers {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        future: ReqFuture<FindBestBuyVesselOfferResult>,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.request_credit_limit_increase_future_seed)])]
    WaitForCreditLimitIncreased {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        future: ReqFuture<RequestCreditLimitIncreaseResult>,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.place_order_future_seed)])]
    WaitForOrderToBeAccepted {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        future: ReqFuture<PlaceBuyCustomVesselOrderResult>,
    },
    ProcessOrder {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        order: WeakBuyCustomVesselOrder,
    },
}

#[derive(Clone)]
pub(crate) struct BuyVesselObjectiveSeed<'context> {
    find_offers_future_seed: ReqFutureSeed<'context, FindBestBuyVesselOfferResult>,
    request_credit_limit_increase_future_seed:
        ReqFutureSeed<'context, RequestCreditLimitIncreaseResult>,
    place_order_future_seed: ReqFutureSeed<'context, PlaceBuyCustomVesselOrderResult>,
}

impl<'context> BuyVesselObjectiveSeed<'context> {
    pub(crate) fn new(context: &'context ReqContext) -> Self {
        Self {
            find_offers_future_seed: ReqFutureSeed::new(context),
            request_credit_limit_increase_future_seed: ReqFutureSeed::new(context),
            place_order_future_seed: ReqFutureSeed::new(context),
        }
    }
}

impl BuyVesselObjective {
    pub(crate) fn new(
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        logger: &mut PersonLogger,
    ) -> Self {
        logger.info(format!(
            "Switched to {}: {:?} {:?}",
            TYPE_ID, needed_capabilities, needed_primary_capabilities
        ));

        Self::CheckPrerequisites {
            needed_capabilities,
            needed_primary_capabilities,
        }
    }
}

impl Objective for BuyVesselObjective {
    type Result = ();
    type Error = BuyVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::CheckPrerequisites {
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                *self = Self::FindOffers {
                    needed_capabilities: needed_capabilities.clone(),
                    needed_primary_capabilities: needed_primary_capabilities.clone(),
                    future: FindBestBuyVesselOffer {
                        prefer_to_buy_from: Some(this_vessel.id()),
                        required_capabilities: std::mem::take(needed_capabilities),
                        required_primary_capabilities: std::mem::take(needed_primary_capabilities),
                    }
                    .push(environment_context.request_storage_mut()),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::FindOffers {
                needed_capabilities,
                needed_primary_capabilities,
                future,
            } => match future.take() {
                Ok(result) => match result {
                    FindBestBuyVesselOfferResult::BuyVesselOffer(_) => todo!(),
                    FindBestBuyVesselOfferResult::BuyCustomVesselOffer { offer, estimate } => {
                        match this_person.ensure_has_money_in_wallet(
                            environment_context.bank_registry(),
                            environment_context.wallet_registry(),
                            estimate.pledge.clone(),
                        ) {
                            Ok(_) => {}
                            Err(WithdrawalError::CreditLimitReached {
                                bank_owner,
                                requested,
                                limit,
                            }) => {
                                *self = Self::WaitForCreditLimitIncreased {
                                    needed_capabilities: std::mem::take(needed_capabilities),
                                    needed_primary_capabilities: std::mem::take(
                                        needed_primary_capabilities,
                                    ),
                                    future: RequestCreditLimitIncrease {
                                        recipient: bank_owner,
                                        new_limit: requested,
                                    }
                                    .push(environment_context.request_storage_mut()),
                                };
                                return Ok(ObjectiveStatus::InProgress);
                            }
                            Err(WithdrawalError::InDebt) => todo!("person: {:?}", this_person),
                        }

                        match person::utils::place_buy_vessel_order(
                            this_person,
                            tie(this_module, this_vessel),
                            environment_context,
                            offer,
                            needed_capabilities.clone(),
                            needed_primary_capabilities.clone(),
                        ) {
                            Ok(order) => {
                                *self = Self::ProcessOrder {
                                    needed_capabilities: std::mem::take(needed_capabilities),
                                    needed_primary_capabilities: std::mem::take(
                                        needed_primary_capabilities,
                                    ),
                                    order,
                                };
                                Ok(ObjectiveStatus::InProgress)
                            }
                            Err(future) => {
                                *self = Self::WaitForOrderToBeAccepted {
                                    needed_capabilities: std::mem::take(needed_capabilities),
                                    needed_primary_capabilities: std::mem::take(
                                        needed_primary_capabilities,
                                    ),
                                    future,
                                };
                                Ok(ObjectiveStatus::InProgress)
                            }
                        }
                    }
                    FindBestBuyVesselOfferResult::None => {
                        Err(BuyVesselObjectiveError::NoBuyOffersFound)
                    }
                },
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::WaitForCreditLimitIncreased {
                needed_capabilities,
                needed_primary_capabilities,
                future,
            } => match future.take() {
                Ok(RequestCreditLimitIncreaseResult::LimitIncreased) => {
                    *self = Self::FindOffers {
                        needed_capabilities: needed_capabilities.clone(),
                        needed_primary_capabilities: needed_primary_capabilities.clone(),
                        future: FindBestBuyVesselOffer {
                            prefer_to_buy_from: Some(this_vessel.id()),
                            required_capabilities: std::mem::take(needed_capabilities),
                            required_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                        }
                        .push(environment_context.request_storage_mut()),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Ok(RequestCreditLimitIncreaseResult::RequestDenied) => {
                    todo!()
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::WaitForOrderToBeAccepted {
                future,
                needed_capabilities,
                needed_primary_capabilities,
            } => match future.take() {
                Ok(PlaceBuyCustomVesselOrderResult::Ok(order)) => {
                    *self = Self::ProcessOrder {
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                        order,
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Ok(PlaceBuyCustomVesselOrderResult::NotEnoughMoneyInWallet) => {
                    logger.err("Not enough money in wallet. Trying again...");
                    *self = Self::CheckPrerequisites {
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Ok(PlaceBuyCustomVesselOrderResult::OfferNotFound) => todo!(),
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::ProcessOrder {
                needed_capabilities,
                needed_primary_capabilities,
                order,
            } => match order.is_completed(environment_context.process_token_context()) {
                Ok(true) => todo!(),
                Ok(false) => Ok(ObjectiveStatus::InProgress),
                Err(ProcessTokenExpiredError) => {
                    logger.warn("Order is dropped. Retrying...");
                    *self = CheckPrerequisites {
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
            },
        }
    }
}

#[derive(Debug)]
pub(crate) enum BuyVesselObjectiveError {
    NoBuyOffersFound,
}

impl Display for BuyVesselObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BuyVesselObjectiveError {}

impl Display for BuyVesselObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuyVesselObjective::CheckPrerequisites { .. } => write!(f, "CheckPrerequisites"),
            BuyVesselObjective::FindOffers { .. } => write!(f, "FindOffers"),
            BuyVesselObjective::WaitForCreditLimitIncreased { .. } => {
                write!(f, "WaitForCreditLimitIncreased")
            }
            BuyVesselObjective::WaitForOrderToBeAccepted { .. } => {
                write!(f, "WaitForOrderToBeAccepted")
            }
            BuyVesselObjective::ProcessOrder { .. } => write!(f, "ProcessOrder"),
        }
    }
}
