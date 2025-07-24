use crate::module::ModuleConsole;
use crate::person::PersonId;
use crate::person::crafting_modules_objective::{
    CraftingModulesObjective, CraftingModulesObjectiveError,
};
use crate::person::crafting_vessels_objective::{
    CraftingVesselsObjective, CraftingVesselsObjectiveError,
};
use crate::person::trading_objective::{TradingObjective, TradingObjectiveError};
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, write};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ObjectiveStatus {
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "objective")]
pub(crate) enum Objective {
    CraftingModules(CraftingModulesObjective),
    CraftingVessels(CraftingVesselsObjective),
    Trading(TradingObjective),
}

impl Objective {
    pub(crate) fn pursue(
        &mut self,
        person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
    ) -> Result<ObjectiveStatus, ObjectiveError> {
        match self {
            Self::CraftingModules(objective) => objective
                .pursue(person, this_module, this_vessel)
                .map_err(ObjectiveError::CraftingModules),
            Self::CraftingVessels(objective) => objective
                .pursue(person, this_module, this_vessel)
                .map_err(ObjectiveError::CraftingVessels),
            Self::Trading(objective) => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ObjectiveError {
    CraftingModules(CraftingModulesObjectiveError),
    CraftingVessels(CraftingVesselsObjectiveError),
    Trading(TradingObjectiveError),
}

impl Display for ObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ObjectiveError {}
