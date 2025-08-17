use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::Intermediate;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/*
    - build a ship
    - search for anomalies
*/

static TYPE_ID: &str = "AdventuringObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum AdventuringObjective {
    SearchShipToBuy,
}

impl AdventuringObjective {
    pub(crate) fn new(logger: &mut PersonLogger) -> Self {
        Self::SearchShipToBuy
    }
}

impl Objective for AdventuringObjective {
    type Error = AdventuringObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum AdventuringObjectiveError {}

impl Display for AdventuringObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for AdventuringObjectiveError {}

pub(crate) struct AdventuringObjectiveDecider;

impl ObjectiveDecider for AdventuringObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if person.passions.contains(&Passion::Adventuring)
            || person.passions.contains(&Passion::Flying)
        {
            logger.info("Manage dockyard station objective decided.");
            Some(Box::new(AdventuringObjective::new(logger)))
        } else {
            None
        }
    }
}

pub(crate) struct AdventuringObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for AdventuringObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        todo!()
    }
}

impl DynSerialize for AdventuringObjective {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}
