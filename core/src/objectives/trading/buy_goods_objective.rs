use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::WeakBuyOrder;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::VesselInternalConsole;
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
pub(crate) enum BuyGoodsObjectiveError {}

impl Display for BuyGoodsObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BuyGoodsObjectiveError {}
