use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessToken};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonInfo, PersonLogger};
use dudes_in_space_api::vessel::{MoveToModuleError, VesselInternalConsole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "build_vessel_objective_stage")]
pub(crate) enum BuildVesselObjective {
    SearchingForDockyard {
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
    },
    MovingToDockyardModule {
        dst: ModuleId,
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
    },
    Building {
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        process_token: Option<ProcessToken>,
    },
    Done,
}

impl BuildVesselObjective {
    pub(crate) fn new(
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
    ) -> Self {
        Self::SearchingForDockyard {
            needed_capabilities,
            needed_primary_capabilities,
        }
    }
}

impl Objective for BuildVesselObjective {
    type Error = BuildVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            BuildVesselObjective::SearchingForDockyard {
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                if person::utils::are_dockyard_components_suitable(
                    this_module.module_storages(),
                    this_module.docking_clamps(),
                    needed_capabilities.clone(),
                    needed_primary_capabilities.clone(),
                ) {
                    logger.info("Moving to dockyard module...");
                    *self = Self::MovingToDockyardModule {
                        dst: this_module.id(),
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                for crafting_module in
                    this_vessel.modules_with_capability(ModuleCapability::Dockyard)
                {
                    if person::utils::are_dockyard_components_suitable(
                        crafting_module.module_storages(),
                        crafting_module.docking_clamps(),
                        needed_capabilities.clone(),
                        needed_primary_capabilities.clone(),
                    ) && crafting_module.free_person_slots_count() > 0
                    {
                        logger.info("Moving to dockyard module...");
                        *self = Self::MovingToDockyardModule {
                            dst: crafting_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(BuildVesselObjectiveError::CanNotFindDockyard)
            }
            BuildVesselObjective::MovingToDockyardModule {
                dst,
                needed_capabilities,
                needed_primary_capabilities,
            } => {
                if *dst == this_module.id() {
                    logger.info("Building vessel...");
                    *self = Self::Building {
                        needed_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_capabilities,
                        )),
                        needed_primary_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_primary_capabilities,
                        )),
                        process_token: None,
                    };
                } else {
                    logger.info("Entering dockyard module...");
                    match this_vessel.move_person_to_module(
                        environment_context.subordination_table(),
                        *this_person.id,
                        *dst,
                    ) {
                        Ok(_) => {}
                        Err(MoveToModuleError::ModuleNotFound) => todo!(),
                        Err(MoveToModuleError::NotEnoughSpace) => {
                            logger.info(
                                "Not enough space in dockyard module. Searching another one...",
                            );
                            *self = Self::SearchingForDockyard {
                                needed_capabilities: std::mem::take(needed_capabilities),
                                needed_primary_capabilities: std::mem::take(
                                    needed_primary_capabilities,
                                ),
                            };
                            return Ok(ObjectiveStatus::InProgress);
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            BuildVesselObjective::Building {
                needed_capabilities,
                needed_primary_capabilities,
                process_token,
            } => match process_token {
                None => {
                    if let Some(modules) = person::utils::find_modules_with_capabilities_in_storages(
                        this_module.module_storages(),
                        needed_capabilities.clone(),
                        needed_primary_capabilities.clone(),
                    ) {
                        *process_token = Some(
                            this_module
                                .dockyard_console_mut()
                                .unwrap()
                                .start(modules)
                                .unwrap(),
                        );
                        Ok(ObjectiveStatus::InProgress)
                    } else {
                        Err(BuildVesselObjectiveError::CanNotFindExpectedModulesUponArrivalInDockyard)
                    }
                }
                Some(process_token) => {
                    if process_token
                        .is_completed(environment_context.process_token_context())
                        .unwrap_or(true)
                    {
                        logger.info("Done building the vessel.");
                        *self = Self::Done;
                        return Ok(ObjectiveStatus::Done);
                    }

                    assert!(this_module.in_progress());
                    logger.info("Waiting for building to complete...");
                    if !this_module.interact() {
                        todo!()
                    } else {
                        Ok(ObjectiveStatus::InProgress)
                    }
                }
            },
            BuildVesselObjective::Done => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum BuildVesselObjectiveError {
    CanNotFindDockyard,
    CanNotFindExpectedModulesUponArrivalInDockyard,
}

impl Display for BuildVesselObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BuildVesselObjectiveError {}
