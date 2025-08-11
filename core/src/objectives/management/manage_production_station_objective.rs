use crate::objectives::crafting::{CraftItemsObjective, CraftModulesObjective};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::ModuleConsole;
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::vessel::VesselConsole;
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};

static TYPE_ID: &str = "ManageProductionStationObjective";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ManageProductionStationObjective {
    ExecuteProduction {
        second_attempt: bool,
        craft_objective: CraftItemsObjective,
    },
    CraftFabricator {
        craft_objective: CraftModulesObjective,
    },
}

impl ManageProductionStationObjective {
    pub(crate) fn new() -> Self {
        Self::ExecuteProduction {
            second_attempt: false,
            craft_objective: CraftItemsObjective::new(
                
            )
        }
    }
}

struct ManageProductionStationObjectiveSeed {}

impl ManageProductionStationObjectiveSeed {
    fn new() -> Self {
        Self {}
    }
}

impl<'de> DeserializeSeed<'de> for ManageProductionStationObjectiveSeed {
    type Value = ManageProductionStationObjective;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ManageProductionStationObjective::deserialize(deserializer)
    }
}

impl Objective for ManageProductionStationObjective {
    type Error = ManageProductionStationObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

pub(crate) struct ManageProductionStationObjectiveDecider;

impl ObjectiveDecider for ManageProductionStationObjectiveDecider {
    fn consider(
        &self,
        person_id: &PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if passions.contains(&Passion::Management) || passions.contains(&Passion::Ruling) {
            logger.info("Manage production station objective decided.");
            Some(Box::new(ManageProductionStationObjective::new()))
        } else {
            None
        }
    }
}

impl DynSerialize for ManageProductionStationObjective {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct ManageProductionStationObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for ManageProductionStationObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let obj: ManageProductionStationObjective =
            from_intermediate_seed(ManageProductionStationObjectiveSeed::new(), &intermediate)
                .map_err(Box::new)?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug)]
pub(crate) enum ManageProductionStationObjectiveError {}

impl Display for ManageProductionStationObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ManageProductionStationObjectiveError {}
