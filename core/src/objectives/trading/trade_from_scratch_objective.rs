use dudes_in_space_api::module::{Module, ModuleCapability, ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{ Display, Formatter};
use serde_intermediate::{from_intermediate, to_intermediate, Intermediate};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use crate::objectives::trading::TradeObjective;

static TYPE_ID: &str = "TradeFromScratchObjective";

static NEEDED_PRIMARY_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::ItemStorage,
];

static NEEDED_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum TradeFromScratchObjective {
    ExecuteTrade { trade_objective: TradeObjective },
    CraftVessel,
}

impl TradeFromScratchObjective {
    pub fn new() -> Self {
        Self::ExecuteTrade { trade_objective: TradeObjective::new() }
    }
}

impl Objective for TradeFromScratchObjective {
    type Error = TradeFromScratchObjectiveError;

    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            TradeFromScratchObjective::ExecuteTrade { trade_objective } => todo!(),
            TradeFromScratchObjective::CraftVessel => todo!(),
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
    ) -> Option<Box<dyn DynObjective>> {
        if passions.contains(&Passion::Trade) || passions.contains(&Passion::Money) {
            Some(Box::new(TradeFromScratchObjective::new()))
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

    fn deserialize(&self, intermediate: Intermediate, this_vault: &DynDeserializeSeedVault<dyn DynObjective>) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
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
