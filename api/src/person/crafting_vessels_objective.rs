use crate::module::{Module, ModuleCapability, ModuleConsole};
use crate::person::PersonId;
use crate::person::building_vessels_objective::{
    BuildingVesselsObjective, BuildingVesselsObjectiveError,
};
use crate::person::crafting_modules_objective::{
    CraftingModulesObjective, CraftingModulesObjectiveError,
};
use crate::person::objective::ObjectiveStatus;
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::cell::RefMut;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "crafting_vessels_objective_stage")]
pub(crate) enum CraftingVesselsObjective {
    CheckingAllPrerequisites {
        needed_capabilities: Vec<ModuleCapability>,
    },
    CraftingDockyard {
        needed_capabilities: Vec<ModuleCapability>,
        crafting_objective: CraftingModulesObjective,
    },
    CraftingVesselModules {
        needed_capabilities: Vec<ModuleCapability>,
        crafting_objective: CraftingModulesObjective,
    },
    BuildingVessel {
        needed_capabilities: Vec<ModuleCapability>,
        building_objective: BuildingVesselsObjective,
    },
    Done,
}

impl CraftingVesselsObjective {
    pub(crate) fn new(needed_capabilities: Vec<ModuleCapability>) -> Self {
        Self::CheckingAllPrerequisites {
            needed_capabilities,
        }
    }

    // pub(crate) fn is_done(&self) -> bool {
    //     todo!()
    // }

    fn find_dockyard_with_suitable_modules_in_storage<'a>(
        dockyards: Vec<RefMut<'a, Box<dyn Module>>>,
        needed_capabilities: &[ModuleCapability],
    ) -> Option<RefMut<'a, Box<dyn Module>>> {
        for mut dockyard in dockyards {
            for storage in dockyard.module_storages() {
                if needed_capabilities
                    .iter()
                    .all(|c| storage.contains_modules_with_cap(*c))
                {
                    return Some(dockyard);
                }
            }
        }
        None
    }

    pub(crate) fn pursue(
        &mut self,
        this_person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
    ) -> Result<ObjectiveStatus, CraftingVesselsObjectiveError> {
        match self {
            Self::CheckingAllPrerequisites {
                needed_capabilities,
            } => {
                let dockyards = this_vessel.modules_with_cap(ModuleCapability::Dockyard);

                if dockyards.is_empty() {
                    *self = Self::CraftingDockyard {
                        needed_capabilities: std::mem::take(needed_capabilities),
                        crafting_objective: CraftingModulesObjective::new(
                            vec![ModuleCapability::Dockyard],
                            true,
                        ),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                let dockyard = Self::find_dockyard_with_suitable_modules_in_storage(
                    dockyards,
                    &needed_capabilities,
                );

                if dockyard.is_none() {
                    *self = Self::CraftingVesselModules {
                        needed_capabilities: needed_capabilities.clone(),
                        crafting_objective: CraftingModulesObjective::new(
                            std::mem::take(needed_capabilities),
                            false,
                        ),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                *self = Self::BuildingVessel {
                    needed_capabilities: needed_capabilities.clone(),
                    building_objective: BuildingVesselsObjective::new(std::mem::take(
                        needed_capabilities,
                    )),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::CraftingDockyard {
                needed_capabilities,
                crafting_objective,
            } => {
                match crafting_objective
                    .pursue(this_person, this_module, this_vessel)
                    .map_err(CraftingVesselsObjectiveError::CraftingDockyard)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        *self = Self::CheckingAllPrerequisites {
                            needed_capabilities: std::mem::take(needed_capabilities),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::CraftingVesselModules {
                needed_capabilities,
                crafting_objective,
            } => {
                match crafting_objective
                    .pursue(this_person, this_module, this_vessel)
                    .map_err(CraftingVesselsObjectiveError::CraftingVesselModules)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        *self = Self::CheckingAllPrerequisites {
                            needed_capabilities: std::mem::take(needed_capabilities),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::BuildingVessel {
                needed_capabilities,
                building_objective,
            } => {
                match building_objective
                    .pursue(this_person, this_module, this_vessel)
                    .map_err(CraftingVesselsObjectiveError::BuildingVessel)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        *self = Self::CheckingAllPrerequisites {
                            needed_capabilities: std::mem::take(needed_capabilities),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Done => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum CraftingVesselsObjectiveError {
    CraftingDockyard(CraftingModulesObjectiveError),
    CraftingVesselModules(CraftingModulesObjectiveError),
    BuildingVessel(BuildingVesselsObjectiveError),
}

impl Display for CraftingVesselsObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftingVesselsObjectiveError {}
