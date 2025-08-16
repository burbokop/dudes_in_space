use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, PersonInfo, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::Intermediate;
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "MineAsteroidsObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MineAsteroidsObjective {}

impl Objective for MineAsteroidsObjective {
    type Error = MineAsteroidsObjectiveError;

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

pub(crate) struct MineAsteroidsObjectiveDecider;

impl ObjectiveDecider for MineAsteroidsObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        None
    }
}

pub(crate) struct MineAsteroidsObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for MineAsteroidsObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum MineAsteroidsObjectiveError {}

impl Display for MineAsteroidsObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MineAsteroidsObjectiveError {}
