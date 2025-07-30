mod common;
mod crafting;
mod gathering;
mod trading;

use crate::objectives::gathering::{
    GatherResearchDataObjectiveDecider, GatherResearchDataObjectiveDynSeed,
    MineAsteroidsObjectiveDecider, MineAsteroidsObjectiveDynSeed, ScavengeObjectiveDecider,
    ScavengeObjectiveDynSeed,
};
use crate::objectives::trading::{
    TradeFromScratchObjectiveDecider, TradeFromScratchObjectiveDynSeed, TradeObjectiveDecider,
    TradeObjectiveDynSeed,
};
use dudes_in_space_api::person::{DynObjective, ObjectiveDeciderVault};
use dyn_serde::DynDeserializeSeedVault;

pub fn register_objectives(
    vault: DynDeserializeSeedVault<dyn DynObjective>,
) -> DynDeserializeSeedVault<dyn DynObjective> {
    vault
        .with(TradeFromScratchObjectiveDynSeed)
        .with(GatherResearchDataObjectiveDynSeed)
        .with(MineAsteroidsObjectiveDynSeed)
        .with(ScavengeObjectiveDynSeed)
}

pub fn register_objective_deciders(vault: ObjectiveDeciderVault) -> ObjectiveDeciderVault {
    vault
        .with(TradeFromScratchObjectiveDecider)
        .with(GatherResearchDataObjectiveDecider)
        .with(MineAsteroidsObjectiveDecider)
        .with(ScavengeObjectiveDecider)
}
