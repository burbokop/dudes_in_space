use crate::objectives::crafting::{
    CraftVesselFromScratchObjective, CraftVesselFromScratchObjectiveError,
};
use crate::objectives::trading::{TradeObjective, TradeObjectiveError, TradeObjectiveSeed};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{from_intermediate_seed, DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::utils::request::{ReqContext};
use dyn_serde_macro::DeserializeSeedXXX;

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
        this_person: PersonId,
        trade_objective: TradeObjective,
    },
    CraftVessel {
        this_person: PersonId,
        craft_vessel_objective: CraftVesselFromScratchObjective,
    },
}

#[derive(Clone)]
struct TradeFromScratchObjectiveSeed<'context> {
    trade_objective_seed: TradeObjectiveSeed<'context>
}

impl<'context> TradeFromScratchObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self { trade_objective_seed: TradeObjectiveSeed::new(context) }
    }
}

impl TradeFromScratchObjective {
    pub fn new(this_person: PersonId) -> Self {
        Self::ExecuteTrade {
            second_attempt: false,
            this_person,
            trade_objective: TradeObjective::new(),
        }
    }
}

impl Objective for TradeFromScratchObjective {
    type Error = TradeFromScratchObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            TradeFromScratchObjective::ExecuteTrade {
                second_attempt,
                this_person,
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
                        this_person: *this_person,
                        craft_vessel_objective: CraftVesselFromScratchObjective::new(
                            this_person.clone(),
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
                this_person,
                craft_vessel_objective,
            } => {
                match craft_vessel_objective
                    .pursue(this_person, this_module, this_vessel, environment_context, logger)
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
                            this_person: this_person.clone(),
                            trade_objective: TradeObjective::SearchVessel {
                            },
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
        person_id: &PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if passions.contains(&Passion::Trade) || passions.contains(&Passion::Money) {
            logger.info("Trade from scratch objective decided.");
            Some(Box::new(TradeFromScratchObjective::new(*person_id)))
        } else {
            None
        }
    }
}

pub(crate) struct TradeFromScratchObjectiveDynSeed{
    req_context: Rc<ReqContext>
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
        let obj: TradeFromScratchObjective = from_intermediate_seed(TradeFromScratchObjectiveSeed::new(&self.req_context), &intermediate).map_err(Box::new)?;
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
