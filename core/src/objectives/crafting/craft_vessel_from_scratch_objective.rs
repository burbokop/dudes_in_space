use crate::objectives::crafting::{
    BuildVesselObjective, BuildVesselObjectiveError, CraftModulesObjective,
    CraftModulesObjectiveError,
};
use dudes_in_space_api::module::{
    ModuleCapability, ModuleConsole, ModuleStorage, ProcessTokenContext,
};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::cell::RefMut;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "crafting_vessels_objective_stage")]
pub(crate) enum CraftVesselFromScratchObjective {
    CheckingAllPrerequisites {
        this_person: PersonId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
    },
    CraftingDockyard {
        this_person: PersonId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        crafting_objective: CraftModulesObjective,
    },
    CraftingVesselModules {
        this_person: PersonId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        crafting_objective: CraftModulesObjective,
    },
    BuildingVessel {
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        building_objective: BuildVesselObjective,
    },
    Done,
}

struct DockyardRef<'x> {
    module_storages: &'x [ModuleStorage],
}
impl CraftVesselFromScratchObjective {
    pub(crate) fn new(
        this_person: PersonId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
    ) -> Self {
        Self::CheckingAllPrerequisites {
            this_person,
            needed_capabilities,
            needed_primary_capabilities,
        }
    }

    fn find_dockyard_with_suitable_modules_in_storage<'a>(
        dockyards: Vec<DockyardRef<'a>>,
        needed_capabilities: &[ModuleCapability],
        needed_primary_capabilities: &[ModuleCapability],
    ) -> Option<DockyardRef<'a>> {
        for mut dockyard in dockyards {
            for storage in dockyard.module_storages {
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
}

impl Objective for CraftVesselFromScratchObjective {
    type Error = CraftVesselFromScratchObjectiveError;

    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CheckingAllPrerequisites {
                this_person,
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                let dockyards = this_vessel.modules_with_cap(ModuleCapability::Dockyard);
                
                let dockyards: Vec<_> = dockyards
                    .iter()
                    .map(|x| DockyardRef {
                        module_storages: x.module_storages(),
                    })
                    .chain( this_module
                        .capabilities()
                        .contains(&ModuleCapability::Dockyard).then_some(DockyardRef {
                        module_storages: this_module.module_storages(),
                    }).into_iter())
                    .collect();

                if dockyards.is_empty() {
                    *self = Self::CraftingDockyard {
                        this_person: this_person.clone(),
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                        crafting_objective: CraftModulesObjective::new(
                            this_person.clone(),
                            vec![ModuleCapability::Dockyard],
                            vec![],
                            true,
                        ),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                let dockyard = Self::find_dockyard_with_suitable_modules_in_storage(
                    dockyards,
                    &needed_capabilities,
                    &needed_primary_capabilities,
                );

                if dockyard.is_none() {
                    *self = Self::CraftingVesselModules {
                        this_person: this_person.clone(),
                        needed_capabilities: needed_capabilities.clone(),
                        needed_primary_capabilities: needed_primary_capabilities.clone(),
                        crafting_objective: CraftModulesObjective::new(
                            this_person.clone(),
                            std::mem::take(needed_capabilities),
                            std::mem::take(needed_primary_capabilities),
                            false,
                        ),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                *self = Self::BuildingVessel {
                    needed_capabilities: needed_capabilities.clone(),
                    needed_primary_capabilities: needed_primary_capabilities.clone(),
                    building_objective: BuildVesselObjective::new(
                        this_person.clone(),
                        std::mem::take(needed_capabilities),
                    ),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::CraftingDockyard {
                this_person,
                needed_capabilities,
                needed_primary_capabilities,
                crafting_objective,
            } => {
                match crafting_objective
                    .pursue(this_module, this_vessel, process_token_context, logger)
                    .map_err(CraftVesselFromScratchObjectiveError::CraftingDockyard)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        *self = Self::CheckingAllPrerequisites {
                            this_person: this_person.clone(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::CraftingVesselModules {
                this_person,
                needed_capabilities,
                needed_primary_capabilities,
                crafting_objective,
            } => {
                match crafting_objective
                    .pursue(this_module, this_vessel, process_token_context, logger)
                    .map_err(CraftVesselFromScratchObjectiveError::CraftingVesselModules)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        *self = Self::CheckingAllPrerequisites {
                            this_person: this_person.clone(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::BuildingVessel {
                needed_capabilities,
                needed_primary_capabilities,
                building_objective,
            } => {
                match building_objective
                    .pursue(this_module, this_vessel, process_token_context, logger)
                    .map_err(CraftVesselFromScratchObjectiveError::BuildingVessel)?
                {
                    ObjectiveStatus::InProgress => Ok(ObjectiveStatus::InProgress),
                    ObjectiveStatus::Done => {
                        *self = Self::Done;
                        Ok(ObjectiveStatus::Done)
                    }
                }
            }
            Self::Done => Ok(ObjectiveStatus::Done),
        }
    }
}

#[derive(Debug)]
pub(crate) enum CraftVesselFromScratchObjectiveError {
    CraftingDockyard(CraftModulesObjectiveError),
    CraftingVesselModules(CraftModulesObjectiveError),
    BuildingVessel(BuildVesselObjectiveError),
}

impl Display for CraftVesselFromScratchObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftVesselFromScratchObjectiveError {}
