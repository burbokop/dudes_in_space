use dudes_in_space_api::environment::EnvironmentContext;

use crate::objectives::common::FlyAndDockToVesselObjective;
use dudes_in_space_api::module::{DockingStatus, ModuleConsole};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::trade::WeakBuyOrder;
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "BuyGoodsObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BuyGoodsObjective {
    order: WeakBuyOrder,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    fly_and_dock_to_vessel_objective: Option<FlyAndDockToVesselObjective>,
}

impl BuyGoodsObjective {
    pub(crate) fn new(order: WeakBuyOrder) -> Self {
        Self {
            order,
            fly_and_dock_to_vessel_objective: None,
        }
    }
}

impl Objective for BuyGoodsObjective {
    type Result = ();
    type Error = BuyGoodsObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        if let Some(fly_and_dock_to_vessel_objective) = &mut self.fly_and_dock_to_vessel_objective {
            match fly_and_dock_to_vessel_objective.pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            ) {
                Ok(ObjectiveStatus::InProgress) => return Ok(ObjectiveStatus::InProgress),
                Ok(ObjectiveStatus::Passive) => return Ok(ObjectiveStatus::Passive),
                Ok(ObjectiveStatus::Done(_)) => self.fly_and_dock_to_vessel_objective = None,
                Err(_) => todo!(),
            }
        }

        let flight_control_console = this_module.flight_control_console_mut().unwrap();
        let target = self
            .order
            .vessel_to_buy_from(environment_context.order_holder())
            .unwrap();

        if flight_control_console.docking_status(environment_context, &target)
            != DockingStatus::Docked
        {
            self.fly_and_dock_to_vessel_objective = Some(FlyAndDockToVesselObjective::new(target));
            return Ok(ObjectiveStatus::InProgress);
        }

        todo!(
            "Transfer wares from station this vessel is docked to this vessel and end the order. Probably it can be done through trading console (Also you should think how ware storages should be integrated in the process)"
        );
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
