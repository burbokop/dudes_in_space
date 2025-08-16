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

static TYPE_ID: &str = "GatherResearchDataObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GatherResearchDataObjective {}

impl Objective for GatherResearchDataObjective {
    type Error = GatherResearchDataObjectiveError;

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

pub(crate) struct GatherResearchDataObjectiveDecider;

impl ObjectiveDecider for GatherResearchDataObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        None
    }
}

pub(crate) struct GatherResearchDataObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for GatherResearchDataObjectiveDynSeed {
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
pub(crate) enum GatherResearchDataObjectiveError {}

impl Display for GatherResearchDataObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GatherResearchDataObjectiveError {}
