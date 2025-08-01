use crate::objectives::crafting::CraftVesselFromScratchObjective;
use crate::objectives::trading::{TradeObjective, TradeObjectiveError};
use dudes_in_space_api::module::{ ModuleCapability, ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "TradeFromScratchObjective";

static NEEDED_PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];

static NEEDED_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum TradeFromScratchObjective {
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
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            TradeFromScratchObjective::ExecuteTrade {
                second_attempt,
                this_person,
                trade_objective,
            } => match trade_objective.pursue(
                this_module,
                this_vessel,
                process_token_context,
                logger,
            ) {
                Ok(ok) => Ok(ok),
                Err(TradeObjectiveError::SuitableVesselNotFound) => {
                    assert!(!*second_attempt);
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
                    .pursue(this_module, this_vessel, process_token_context, logger)
                    .map_err(|_| {
                        TradeFromScratchObjectiveError::SuitableVesselNotFoundAndCanNotBeCrafted
                    })? {
                    ObjectiveStatus::InProgress => Ok(ObjectiveStatus::InProgress),
                    ObjectiveStatus::Done => {
                        logger.info("ExecuteTrade");
                        *self = TradeFromScratchObjective::ExecuteTrade {
                            second_attempt: true,
                            this_person: this_person.clone(),
                            trade_objective: TradeObjective::SearchVessel,
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
        person_id: PersonId,
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
            Some(Box::new(TradeFromScratchObjective::new(person_id)))
        } else {
            None
        }
    }
}

pub(crate) struct TradeFromScratchObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for TradeFromScratchObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let obj: TradeFromScratchObjective = from_intermediate(&intermediate).map_err(Box::new)?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug)]
pub(crate) enum TradeFromScratchObjectiveError {
    SuitableVesselNotFoundAndCanNotBeCrafted,
}

impl Display for TradeFromScratchObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TradeFromScratchObjectiveError {}
