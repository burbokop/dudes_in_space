use dudes_in_space_api::module::{ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use serde_intermediate::Intermediate;
use dyn_serde::{DynSerialize, TypeId};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum TradeObjective {
    SearchForBuyOffers,
    MoveToVesselToBuy,
    SearchForSellOffers,
    MoveToVesselToSell,
}

impl TradeObjective {
    pub fn new() -> Self {
        Self::SearchForBuyOffers
    }
}

impl Objective for TradeObjective {
    type Error = TradeObjectiveError;

    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            TradeObjective::SearchForBuyOffers => todo!(),
            TradeObjective::MoveToVesselToBuy => todo!(),
            TradeObjective::SearchForSellOffers => todo!(),
            TradeObjective::MoveToVesselToSell => todo!(),
        }
    }
}

impl DynSerialize for TradeObjective {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

pub(crate) struct TradeObjectiveDecider;

impl ObjectiveDecider for TradeObjectiveDecider {
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
            Some(Box::new(TradeObjective::new()))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) enum TradeObjectiveError {}

impl Display for TradeObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for TradeObjectiveError {}
