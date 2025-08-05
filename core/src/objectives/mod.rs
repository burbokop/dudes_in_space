mod common;
mod crafting;
mod gathering;
mod trading;

use std::rc::Rc;
use crate::objectives::gathering::{
    GatherResearchDataObjectiveDecider, GatherResearchDataObjectiveDynSeed,
    MineAsteroidsObjectiveDecider, MineAsteroidsObjectiveDynSeed, ScavengeObjectiveDecider,
    ScavengeObjectiveDynSeed,
};
use crate::objectives::trading::{
    TradeFromScratchObjectiveDecider, TradeFromScratchObjectiveDynSeed,
};
use dudes_in_space_api::person::{DynObjective, ObjectiveDeciderVault};
use dudes_in_space_api::utils::request::ReqContext;
use dyn_serde::DynDeserializeSeedVault;

pub fn register_objectives(
    vault: DynDeserializeSeedVault<dyn DynObjective>,
    req_context: Rc<ReqContext>,
) -> DynDeserializeSeedVault<dyn DynObjective> {
    vault
        .with(TradeFromScratchObjectiveDynSeed::new(req_context))
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
