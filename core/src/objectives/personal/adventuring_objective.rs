use dudes_in_space_api::environment::{
    EnvironmentContext, FindOwnedVessels, FindOwnedVesselsResult,
};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

/*
    - build a ship
    - search for anomalies
*/

static TYPE_ID: &str = "AdventuringObjective";
static NEEDED_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "adventuring_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::personal::adventuring_objective::AdventuringObjectiveSeed::<'context>)]
pub(crate) enum AdventuringObjective {
    CheckThisVessel,
    MoveToCockpit {
        dst: ModuleId,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    SearchForOwnedShips {
        future: ReqFuture<FindOwnedVesselsResult>,
    },
    SearchShipToBuy,
    Fly,
}

struct AdventuringObjectiveSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindOwnedVesselsResult>,
}

impl<'context> AdventuringObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            req_future_seed: ReqFutureSeed::new(context),
        }
    }
}

impl AdventuringObjective {
    pub(crate) fn new(logger: &mut PersonLogger) -> Self {
        Self::CheckThisVessel
    }
}

impl Objective for AdventuringObjective {
    type Error = AdventuringObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            AdventuringObjective::CheckThisVessel => {
                if &this_vessel.owner() == this_person.id {
                    for module in this_vessel.modules_with_capability(ModuleCapability::Cockpit) {
                        if module.free_person_slots_count() > 0 {
                            logger.info("Moving to dockyard module...");
                            *self = Self::MoveToCockpit { dst: module.id() };
                            return Ok(ObjectiveStatus::InProgress);
                        }
                    }

                    todo!("No cockpit module found.")
                }

                *self = Self::SearchForOwnedShips {
                    future: FindOwnedVessels {
                        owner: *this_person.id,
                        required_capabilities: NEEDED_CAPABILITIES.iter().cloned().collect(),
                    }
                    .push(environment_context.request_storage_mut()),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            AdventuringObjective::MoveToCockpit { .. } => todo!(),
            AdventuringObjective::SearchForOwnedShips { future } => match future.take() {
                Ok(search_result) => todo!("Search for other owned vessels."),
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            AdventuringObjective::SearchShipToBuy => todo!(),
            AdventuringObjective::Fly => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum AdventuringObjectiveError {}

impl Display for AdventuringObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for AdventuringObjectiveError {}

pub(crate) struct AdventuringObjectiveDecider;

impl ObjectiveDecider for AdventuringObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if person.passions.contains(&Passion::Adventuring)
            || person.passions.contains(&Passion::Flying)
        {
            logger.info("Manage dockyard station objective decided.");
            Some(Box::new(AdventuringObjective::new(logger)))
        } else {
            None
        }
    }
}

pub(crate) struct AdventuringObjectiveDynSeed {
    req_context: Rc<ReqContext>,
}

impl AdventuringObjectiveDynSeed {
    pub(crate) fn new(req_context: Rc<ReqContext>) -> Self {
        Self { req_context }
    }
}

impl DynDeserializeSeed<dyn DynObjective> for AdventuringObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let r: AdventuringObjective = from_intermediate_seed(
            AdventuringObjectiveSeed::new(&self.req_context),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(r))
    }
}

impl DynSerialize for AdventuringObjective {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}
