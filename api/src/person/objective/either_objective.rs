use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::PersonId;
use crate::person::objective::crafting::{
    CraftModulesObjective, CraftModulesObjectiveError, CraftVesselFromScratchObjective,
    CraftVesselFromScratchObjectiveError,
};
use crate::person::objective::trading::{TradeObjective, TradeObjectiveError};
use crate::person::objective::{
    AdventuringObjective, AdventuringObjectiveError, Objective, ObjectiveStatus,
};
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "objective")]
pub(crate) enum EitherObjective {
    CraftingModules(CraftModulesObjective),
    CraftingVessels(CraftVesselFromScratchObjective),
    Trading(TradeObjective),
    Adventuring(AdventuringObjective),
}

impl Objective for EitherObjective {
    type Error = EitherObjectiveError;

    fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CraftingModules(objective) => objective
                .pursue(person, this_module, this_vessel, process_token_context)
                .map_err(EitherObjectiveError::CraftingModules),
            Self::CraftingVessels(objective) => objective
                .pursue(person, this_module, this_vessel, process_token_context)
                .map_err(EitherObjectiveError::CraftingVessels),
            Self::Trading(objective) => objective
                .pursue(person, this_module, this_vessel, process_token_context)
                .map_err(EitherObjectiveError::Trading),
            Self::Adventuring(objective) => objective
                .pursue(person, this_module, this_vessel, process_token_context)
                .map_err(EitherObjectiveError::Adventuring),
        }
    }
}

#[derive(Debug)]
pub(crate) enum EitherObjectiveError {
    CraftingModules(CraftModulesObjectiveError),
    CraftingVessels(CraftVesselFromScratchObjectiveError),
    Trading(TradeObjectiveError),
    Adventuring(AdventuringObjectiveError),
}

impl Display for EitherObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for EitherObjectiveError {}
