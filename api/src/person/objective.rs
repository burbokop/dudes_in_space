use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PointeeSized;
use serde::{Deserialize, Serialize};
use crate::modules::{ModulePersonInterface, VesselPersonInterface};
use crate::{Person, PersonId};
use crate::person::crafting_modules_objective::{CraftingModulesObjective, CraftingModulesObjectiveError};
use crate::person::crafting_vessels_objective::{CraftingVesselsObjective, CraftingVesselsObjectiveError};
use crate::person::trading_objective::{TradingObjective, TradingObjectiveError};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ObjectiveStatus {
    InProgress,
    Done
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "objective")]
pub(crate) enum Objective {
    CraftingModules(CraftingModulesObjective),
    CraftingVessels(CraftingVesselsObjective),
    Trading(TradingObjective),
}

impl Objective {
    pub(crate) fn pursue(&mut self, person: PersonId, this_module: &mut dyn ModulePersonInterface, this_vessel: &dyn VesselPersonInterface) -> Result<ObjectiveStatus, ObjectiveError> {
        match self {
            Self::CraftingModules(objective) => objective.pursue(person,this_module,this_vessel).map_err(ObjectiveError::CraftingModules),
            Self::CraftingVessels(objective) => objective.pursue(person,this_module,this_vessel).map_err(ObjectiveError::CraftingVessels),
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
        todo!()
    }
}

impl Error for ObjectiveError {}
