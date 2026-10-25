use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{DockingStatus, ModuleCapability, ModuleConsole};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{VesselId, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "FlyAndDockToVesselObjective";
static DISTANCE_EPSILON: Float = 0.000001;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FlyAndDockToVesselObjective {
    target: VesselId,
}

impl FlyAndDockToVesselObjective {
    pub(crate) fn new(target: VesselId) -> Self {
        Self { target }
    }
}

impl Objective for FlyAndDockToVesselObjective {
    type Result = ();
    type Error = FlyAndDockToVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        assert!(
            this_module
                .capabilities()
                .contains(&ModuleCapability::FlightControl)
        );
        assert!(
            this_module
                .capabilities()
                .contains(&ModuleCapability::Navigation)
        );

        // Is docked to another vessel
        if this_module
            .flight_control_console_mut()
            .unwrap()
            .is_docked()
            && this_module
                .flight_control_console_mut()
                .unwrap()
                .docking_status(environment_context, &self.target)
                != DockingStatus::Docked
        {
            if !this_module
                .flight_control_console_mut()
                .unwrap()
                .can_undock()
            {
                return Ok(ObjectiveStatus::Passive);
            }

            this_module
                .flight_control_console_mut()
                .unwrap()
                .undock()
                .unwrap();
            return Ok(ObjectiveStatus::InProgress);
        }

        match this_module
            .flight_control_console_mut()
            .unwrap()
            .docking_status(environment_context, &self.target)
        {
            DockingStatus::Docked => Ok(ObjectiveStatus::Done(())),
            DockingStatus::CanDock => {
                this_module
                    .flight_control_console_mut()
                    .unwrap()
                    .dock(&self.target)
                    .unwrap();
                return Ok(ObjectiveStatus::Done(()));
            }
            DockingStatus::NoAvailableDockingPorts | DockingStatus::TooFar => {
                let navigation_console = this_module.navigation_console().unwrap();
                let vector = navigation_console.vector_to_vessel(&self.target);

                if vector.len() < DISTANCE_EPSILON {
                    return Ok(ObjectiveStatus::Passive);
                }

                this_module
                    .flight_control_console_mut()
                    .unwrap()
                    .fly(vector)
                    .unwrap();

                return Ok(ObjectiveStatus::InProgress);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum FlyAndDockToVesselObjectiveError {}

impl Display for FlyAndDockToVesselObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for FlyAndDockToVesselObjectiveError {}
