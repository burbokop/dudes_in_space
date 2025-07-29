use dudes_in_space_api::module::{ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use serde_intermediate::Intermediate;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, TypeId};

static TYPE_ID: &str = "GatherResearchDataObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GatherResearchDataObjective {}

impl Objective for GatherResearchDataObjective {
    type Error = GatherResearchDataObjectiveError;

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

pub(crate) struct GatherResearchDataObjectiveDecider;

impl ObjectiveDecider for GatherResearchDataObjectiveDecider {
    fn consider(
        &self,
        person_id: PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
    ) -> Option<Box<dyn DynObjective>> {
        None
    }
}


pub(crate) struct GatherResearchDataObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for GatherResearchDataObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(&self, intermediate: Intermediate, this_vault: &DynDeserializeSeedVault<dyn DynObjective>) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
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
