use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::PersonId;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum TradeObjective {
    SearchForBuyOffers,
    MoveToVesselToBuy,
    SearchForSellOffers,
    MoveToVesselToSell,
}

impl Objective for TradeObjective {
    type Error = TradeObjectiveError;

    fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            TradeObjective::SearchForBuyOffers => todo!(),
            TradeObjective::MoveToVesselToBuy => todo!(),
            TradeObjective::SearchForSellOffers => todo!(),
            TradeObjective::MoveToVesselToSell => todo!(),
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
