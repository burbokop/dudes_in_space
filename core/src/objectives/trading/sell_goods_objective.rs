use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::WeakSellOrder;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::VesselInternalConsole;
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
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
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
