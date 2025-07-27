use dudes_in_space_api::item::WeakSellOrder;
use dudes_in_space_api::module::{ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SellGoodsObjective {
    order: WeakSellOrder,
}

impl SellGoodsObjective {
    pub(crate) fn new(order: WeakSellOrder) -> Self {
        Self { order }
    }
}

impl Objective for SellGoodsObjective {
    type Error = SellGoodsObjectiveError;

    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
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
