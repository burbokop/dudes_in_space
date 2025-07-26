use crate::module::{
    ModuleCapability, ModuleConsole, ModuleId, ModuleStorage, ProcessToken, ProcessTokenContext,
};
use crate::person::PersonId;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "building_vessels_objective_stage")]
pub(crate) enum BuildVesselObjective {
    SearchingForDockyard {
        needed_capabilities: Vec<ModuleCapability>,
    },
    MovingToDockyardModule {
        dst: ModuleId,
        needed_capabilities: Vec<ModuleCapability>,
    },
    Building {
        needed_capabilities: BTreeSet<ModuleCapability>,
        process_token: Option<ProcessToken>,
    },
    Done,
}

impl BuildVesselObjective {
    pub(crate) fn new(needed_capabilities: Vec<ModuleCapability>) -> Self {
        Self::SearchingForDockyard {
            needed_capabilities,
        }
    }

    fn are_module_storages_suitable(
        storages: &[ModuleStorage],
        needed_capabilities: Vec<ModuleCapability>,
    ) -> bool {
        storages.iter().any(|storage| {
            let mut needed_capabilities = needed_capabilities.clone();
            for module in storage.iter() {
                for cap in module.capabilities() {
                    if let Some(i) = needed_capabilities.iter().position(|x| *x == *cap) {
                        needed_capabilities.remove(i);
                    }
                }
            }
            needed_capabilities.is_empty()
        })
    }

    fn find_modules_with_capabilities(
        storages: &[ModuleStorage],
        needed_capabilities: BTreeSet<ModuleCapability>,
    ) -> Option<BTreeSet<ModuleId>> {
        for storage in storages {
            let mut needed_capabilities = needed_capabilities.clone();

            let mut modules: BTreeSet<ModuleId> = Default::default();
            for module in storage.iter() {
                for cap in module.capabilities() {
                    if needed_capabilities.contains(cap) {
                        modules.insert(module.id());
                        needed_capabilities.remove(cap);
                    }
                }
            }

            if needed_capabilities.is_empty() {
                return Some(modules);
            }
        }
        None
    }
}

impl Objective for BuildVesselObjective {
    type Error = BuildVesselObjectiveError;

    fn pursue(
        &mut self,
        this_person: PersonId,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            BuildVesselObjective::SearchingForDockyard {
                needed_capabilities,
            } => {
                if Self::are_module_storages_suitable(
                    this_module.module_storages(),
                    needed_capabilities.clone(),
                ) {
                    *self = Self::MovingToDockyardModule {
                        dst: this_module.id(),
                        needed_capabilities: std::mem::take(needed_capabilities),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                for mut crafting_module in this_vessel.modules_with_cap(ModuleCapability::Dockyard)
                {
                    if Self::are_module_storages_suitable(
                        crafting_module.module_storages(),
                        needed_capabilities.clone(),
                    ) {
                        *self = Self::MovingToDockyardModule {
                            dst: crafting_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                        };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                Err(BuildVesselObjectiveError::CanNotFindDockyard)
            }
            BuildVesselObjective::MovingToDockyardModule {
                dst,
                needed_capabilities,
            } => {
                if *dst == this_module.id() {
                    *self = Self::Building {
                        needed_capabilities: BTreeSet::from_iter(std::mem::take(
                            needed_capabilities,
                        )),
                        process_token: None,
                    };
                } else {
                    this_vessel.move_to_module(this_person, *dst);
                }
                Ok(ObjectiveStatus::InProgress)
            }
            BuildVesselObjective::Building {
                needed_capabilities,
                process_token,
            } => match process_token {
                None => {
                    if let Some(modules) = Self::find_modules_with_capabilities(
                        this_module.module_storages(),
                        needed_capabilities.clone(),
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
                        .is_completed(process_token_context)
                        .unwrap_or(true)
                    {
                        *self = Self::Done;
                        return Ok(ObjectiveStatus::Done);
                    }

                    assert!(this_module.in_progress());
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
