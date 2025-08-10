use crate::objectives::crafting::{
    BuildVesselObjective, BuildVesselObjectiveError, CraftModulesObjective,
    CraftModulesObjectiveError,
};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleStorage};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonId, PersonLogger};
use dudes_in_space_api::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "craft_vessel_from_scratch_objective_stage")]
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
        for dockyard in dockyards {
            for storage in dockyard.module_storages {
                if needed_primary_capabilities
                    .iter()
                    .all(|c| storage.contains_modules_with_primary_capability(*c))
                    && needed_capabilities
                        .iter()
                        .all(|c| storage.contains_modules_with_capability(*c))
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
        this_person: &PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::CheckingAllPrerequisites {
                this_person,
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                let dockyards = this_vessel.modules_with_capability(ModuleCapability::Dockyard);

                let dockyards: Vec<_> = dockyards
                    .iter()
                    .map(|x| DockyardRef {
                        module_storages: x.module_storages(),
                    })
                    .chain(
                        this_module
                            .capabilities()
                            .contains(&ModuleCapability::Dockyard)
                            .then_some(DockyardRef {
                                module_storages: this_module.module_storages(),
                            })
                            .into_iter(),
                    )
                    .collect();

                if dockyards.is_empty() {
                    logger.info("Crafting dockyard...");
                    *self = Self::CraftingDockyard {
                        this_person: this_person.clone(),
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                        crafting_objective: CraftModulesObjective::new(
                            this_person.clone(),
                            vec![ModuleCapability::Dockyard],
                            vec![],
                            true,
                            logger,
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
                    logger.info("Crafting modules for a new vessel...");
                    *self = Self::CraftingVesselModules {
                        this_person: this_person.clone(),
                        needed_capabilities: needed_capabilities.clone(),
                        needed_primary_capabilities: needed_primary_capabilities.clone(),
                        crafting_objective: CraftModulesObjective::new(
                            this_person.clone(),
                            std::mem::take(needed_capabilities),
                            std::mem::take(needed_primary_capabilities),
                            false,
                            logger,
                        ),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                logger.info("Beginning vessel building stage...");
                *self = Self::BuildingVessel {
                    needed_capabilities: needed_capabilities.clone(),
                    needed_primary_capabilities: needed_primary_capabilities.clone(),
                    building_objective: BuildVesselObjective::new(
                        this_person.clone(),
                        std::mem::take(needed_capabilities),
                        std::mem::take(needed_primary_capabilities),
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
                    .pursue(
                        this_person,
                        this_module,
                        this_vessel,
                        environment_context,
                        logger,
                    )
                    .map_err(CraftVesselFromScratchObjectiveError::CraftingDockyard)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        logger.info("CraftVesselFromScratchObjective::CraftingDockyard::CheckingAllPrerequisites");
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
                    .pursue(
                        this_person,
                        this_module,
                        this_vessel,
                        environment_context,
                        logger,
                    )
                    .map_err(CraftVesselFromScratchObjectiveError::CraftingVesselModules)?
                {
                    ObjectiveStatus::InProgress => {}
                    ObjectiveStatus::Done => {
                        logger.info(
                            "Checking all prerequisites for crafting a vessel from scratch...",
                        );
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
                    .pursue(
                        this_person,
                        this_module,
                        this_vessel,
                        environment_context,
                        logger,
                    )
                    .map_err(CraftVesselFromScratchObjectiveError::BuildingVessel)?
                {
                    ObjectiveStatus::InProgress => Ok(ObjectiveStatus::InProgress),
                    ObjectiveStatus::Done => {
                        logger.info("Done crafting a vessel from scratch.");
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
