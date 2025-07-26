use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::PersonId;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::item::{WeakBuyOrder, WeakSellOrder};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SellGoodsObjective {
    order: WeakSellOrder,
}

impl SellGoodsObjective {
    pub(crate) fn new(order: WeakSellOrder) -> Self {
        Self { order }
    }
}

impl Objective for crate::person::objective::trading::SellGoodsObjective {
    type Error = SellGoodsObjectiveError;

    fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum SellGoodsObjectiveError {}

impl Display for SellGoodsObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for SellGoodsObjectiveError {}
