use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::utils::request::ReqContext;
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde_intermediate::Intermediate;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/*
    - Find available crafts across all crafters
        - get list of all recipes of all crafters awailable to assemble
        - exclude recipes that are impossible to craft due to input resource missing on the market
        - find best offers for each remaining recipe
        - choose recipe with max profit
    - Craft chosen crafter
    - Place sell offer
    - Craft item
    - Place buy offer
*/

static TYPE_ID: &str = "ManageDockyardStationObjective";

#[derive(Debug)]
enum ManageDockyardStationObjective {
    DecideWhatToDo,
}

impl ManageDockyardStationObjective {
    pub(crate) fn new(logger: &mut PersonLogger) -> Self {
        Self::DecideWhatToDo
    }
}

impl Objective for ManageDockyardStationObjective {
    type Error = ManageDockyardStationObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
enum ManageDockyardStationObjectiveError {}

impl Display for ManageDockyardStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ManageDockyardStationObjectiveError {}

pub(crate) struct ManageDockyardStationObjectiveDecider;

impl ObjectiveDecider for ManageDockyardStationObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if person.passions.contains(&Passion::Management)
            || person.passions.contains(&Passion::Ruling)
        {
            logger.info("Manage dockyard station objective decided.");
            Some(Box::new(ManageDockyardStationObjective::new(logger)))
        } else {
            None
        }
    }
}

pub(crate) struct ManageDockyardStationObjectiveDynSeed {
    req_context: Rc<ReqContext>,
}

impl ManageDockyardStationObjectiveDynSeed {
    pub(crate) fn new(req_context: Rc<ReqContext>) -> Self {
        Self { req_context }
    }
}

impl DynDeserializeSeed<dyn DynObjective> for ManageDockyardStationObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        todo!()
    }
}

impl DynSerialize for ManageDockyardStationObjective {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}
