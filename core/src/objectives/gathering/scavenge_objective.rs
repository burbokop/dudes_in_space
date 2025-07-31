use dudes_in_space_api::module::{ModuleConsole, ProcessTokenContext};
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::Intermediate;
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "MineAsteroidsObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ScavengeObjective {}

impl Objective for ScavengeObjective {
    type Error = ScavengeObjectiveError;

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

pub(crate) struct ScavengeObjectiveDecider;

impl ObjectiveDecider for ScavengeObjectiveDecider {
    fn consider(
        &self,
        person_id: PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        None
    }
}

pub(crate) struct ScavengeObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for ScavengeObjectiveDynSeed {
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
pub(crate) enum ScavengeObjectiveError {}

impl Display for ScavengeObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ScavengeObjectiveError {}
