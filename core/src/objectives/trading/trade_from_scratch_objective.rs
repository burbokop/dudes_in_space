use crate::objectives::crafting::{
    CraftVesselFromScratchObjective, CraftVesselFromScratchObjectiveError,
};
use crate::objectives::trading::{TradeObjective, TradeObjectiveError, TradeObjectiveSeed};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::utils::request::ReqContext;
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

static TYPE_ID: &str = "TradeFromScratchObjective";

static NEEDED_PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];

static NEEDED_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "trade_from_scratch_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::trading::trade_from_scratch_objective::TradeFromScratchObjectiveSeed::<'context>)]
pub(crate) enum TradeFromScratchObjective {
    #[deserialize_seed_xxx(seeds = [(trade_objective, self.seed.seed.trade_objective_seed)])]
    ExecuteTrade {
        second_attempt: bool,
        trade_objective: TradeObjective,
    },
    CraftVessel {
        craft_vessel_objective: CraftVesselFromScratchObjective,
    },
}

#[derive(Clone)]
struct TradeFromScratchObjectiveSeed<'context> {
    trade_objective_seed: TradeObjectiveSeed<'context>,
}

impl<'context> TradeFromScratchObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            trade_objective_seed: TradeObjectiveSeed::new(context),
        }
    }
}

impl TradeFromScratchObjective {
    pub fn new() -> Self {
        Self::ExecuteTrade {
            second_attempt: false,
            trade_objective: TradeObjective::new(),
        }
    }
}

impl Objective for TradeFromScratchObjective {
    type Error = TradeFromScratchObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            TradeFromScratchObjective::ExecuteTrade {
                second_attempt,

                trade_objective,
            } => match trade_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ok) => Ok(ok),
                Err(TradeObjectiveError::SuitableVesselNotFound) => {
                    if *second_attempt {
                        return Err(
                            TradeFromScratchObjectiveError::SuitableVesselNotFoundAfterCrafting,
                        );
                    }

                    logger.info("No suitable vessel found for trade. Crafting it...");
                    *self = Self::CraftVessel {
                        craft_vessel_objective: CraftVesselFromScratchObjective::new(
                            vec![
                                ModuleCapability::Cockpit,
                                ModuleCapability::Engine,
                                ModuleCapability::Reactor,
                                ModuleCapability::FuelTank,
                            ],
                            vec![ModuleCapability::ItemStorage],
                        ),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
            },
            TradeFromScratchObjective::CraftVessel {
                craft_vessel_objective,
            } => {
                match craft_vessel_objective
                    .pursue(
                        this_person,
                        this_module,
                        this_vessel,
                        environment_context,
                        logger,
                    )
                    .map_err(|err| {
                        TradeFromScratchObjectiveError::SuitableVesselNotFoundAndCanNotBeCrafted {
                            reason_why_can_not_be_crafted: err,
                        }
                    })? {
                    ObjectiveStatus::InProgress => Ok(ObjectiveStatus::InProgress),
                    ObjectiveStatus::Done => {
                        logger.info("ExecuteTrade");
                        *self = TradeFromScratchObjective::ExecuteTrade {
                            second_attempt: true,

                            trade_objective: TradeObjective::SearchVessel {},
                        };
                        Ok(ObjectiveStatus::InProgress)
                    }
                }
            }
        }
    }
}

impl DynSerialize for TradeFromScratchObjective {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct TradeFromScratchObjectiveDecider;

impl ObjectiveDecider for TradeFromScratchObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if person.passions.contains(&Passion::Trade) || person.passions.contains(&Passion::Money) {
            logger.info("Trade from scratch objective decided.");
            Some(Box::new(TradeFromScratchObjective::new()))
        } else {
            None
        }
    }
}

pub(crate) struct TradeFromScratchObjectiveDynSeed {
    req_context: Rc<ReqContext>,
}

impl TradeFromScratchObjectiveDynSeed {
    pub fn new(req_context: Rc<ReqContext>) -> Self {
        Self { req_context }
    }
}

impl DynDeserializeSeed<dyn DynObjective> for TradeFromScratchObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let obj: TradeFromScratchObjective = from_intermediate_seed(
            TradeFromScratchObjectiveSeed::new(&self.req_context),
            &intermediate,
        )
        .map_err(Box::new)?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug)]
pub(crate) enum TradeFromScratchObjectiveError {
    SuitableVesselNotFoundAndCanNotBeCrafted {
        reason_why_can_not_be_crafted: CraftVesselFromScratchObjectiveError,
    },
    SuitableVesselNotFoundAfterCrafting,
}

impl Display for TradeFromScratchObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TradeFromScratchObjectiveError {}
