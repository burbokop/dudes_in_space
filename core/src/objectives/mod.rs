mod crafting;
mod gathering;
mod trading;

use crate::objectives::gathering::{
    GatherResearchDataObjectiveDecider, MineAsteroidsObjectiveDecider, ScavengeObjectiveDecider,
};
use crate::objectives::trading::TradeObjectiveDecider;
use dudes_in_space_api::person::{DynObjective, ObjectiveDeciderVault};
use dyn_serde::DynDeserializeSeedVault;

pub fn register_objectives(
    vault: DynDeserializeSeedVault<dyn DynObjective>,
) -> DynDeserializeSeedVault<dyn DynObjective> {
    vault
}

pub fn register_objective_deciders(vault: ObjectiveDeciderVault) -> ObjectiveDeciderVault {
    vault
        .with(TradeObjectiveDecider)
        .with(GatherResearchDataObjectiveDecider)
        .with(MineAsteroidsObjectiveDecider)
        .with(ScavengeObjectiveDecider)
}
