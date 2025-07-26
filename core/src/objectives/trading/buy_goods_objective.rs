use crate::item::{Item, WeakBuyOrder};
use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::PersonId;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::vessel::{VesselConsole, VesselId};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BuyGoodsObjective {
    order: WeakBuyOrder,
}

impl BuyGoodsObjective {
    pub(crate) fn new(order: WeakBuyOrder) -> Self {
        Self { order }
    }
}

impl Objective for BuyGoodsObjective {
    type Error = BuyGoodsObjectiveError;

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
pub(crate) enum BuyGoodsObjectiveError {}

impl Display for BuyGoodsObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BuyGoodsObjectiveError {}
