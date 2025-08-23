use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestBuyVesselOffer, FindBestBuyVesselOfferResult,
};

use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::trade::WeakBuyVesselOrder;
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
#[deserialize_seed_xxx(seed = crate::objectives::trading::buy_vessel_objective::BuyVesselObjectiveSeed::<'context>)]
pub(crate) enum BuyVesselObjective {
    CheckPrerequisites {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    FindOffers {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        future: ReqFuture<FindBestBuyVesselOfferResult>,
    },
    ProcessOrder {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        order: WeakBuyVesselOrder,
    },
}

#[derive(Clone)]
pub(crate) struct BuyVesselObjectiveSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindBestBuyVesselOfferResult>,
}

impl<'context> BuyVesselObjectiveSeed<'context> {
    pub(crate) fn new(context: &'context ReqContext) -> Self {
        Self {
            req_future_seed: ReqFutureSeed::new(context),
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
    type Error = BuyVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
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
                    FindBestBuyVesselOfferResult::BuyCustomVesselOffer(_) => todo!(),
                    FindBestBuyVesselOfferResult::None => {
                        Err(BuyVesselObjectiveError::NoBuyOffersFound)
                    }
                },
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::ProcessOrder { .. } => todo!(),
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
