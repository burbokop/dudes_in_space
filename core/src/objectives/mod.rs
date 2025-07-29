mod crafting;
mod gathering;
mod trading;
mod common;

use crate::objectives::gathering::{GatherResearchDataObjectiveDecider, GatherResearchDataObjectiveDynSeed, MineAsteroidsObjectiveDecider, MineAsteroidsObjectiveDynSeed, ScavengeObjectiveDecider, ScavengeObjectiveDynSeed};
use crate::objectives::trading::{TradeObjectiveDecider, TradeObjectiveDynSeed};
use dudes_in_space_api::person::{DynObjective, ObjectiveDeciderVault};
use dyn_serde::DynDeserializeSeedVault;

pub fn register_objectives(
    vault: DynDeserializeSeedVault<dyn DynObjective>,
) -> DynDeserializeSeedVault<dyn DynObjective> {
    vault
        .with(TradeObjectiveDynSeed)
        .with(GatherResearchDataObjectiveDynSeed)
        .with(MineAsteroidsObjectiveDynSeed)
        .with(ScavengeObjectiveDynSeed)
}

pub fn register_objective_deciders(vault: ObjectiveDeciderVault) -> ObjectiveDeciderVault {
    vault
        .with(TradeObjectiveDecider)
        .with(GatherResearchDataObjectiveDecider)
        .with(MineAsteroidsObjectiveDecider)
        .with(ScavengeObjectiveDecider)
}
