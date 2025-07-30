use dudes_in_space_api::item::WeakBuyOrder;
use dudes_in_space_api::module::{ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "BuyGoodsObjective";

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
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: &mut PersonLogger,
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
