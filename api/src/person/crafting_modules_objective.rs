use std::collections::{BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PointeeSized;
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use crate::modules::{AssemblyRecipe, ModuleCapability, ModuleId, ModulePersonInterface, VesselPersonInterface};
use crate::{Person, PersonId};
use crate::person::crafting_modules_objective::CraftingModulesObjective::Done;
use crate::person::objective::ObjectiveStatus;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "crafting_modules_objective_stage")]
pub(crate) enum CraftingModulesObjective {
    SearchingForCraftingModule {
        needed_capabilities: Vec<ModuleCapability>,
        deploy: bool,
    },
    MovingToCraftingModule {
        dst: ModuleId,
        needed_capabilities: Vec<ModuleCapability>,
        deploy: bool,
    },
    Crafting {
        needed_capabilities: BTreeSet<ModuleCapability>,
        deploy: bool,
    },
    Done,
}

impl CraftingModulesObjective {
    pub(crate) fn new(needed_capabilities: Vec<ModuleCapability>, deploy: bool) -> Self {
        Self::SearchingForCraftingModule { needed_capabilities, deploy }
    }
    
    pub(crate) fn is_done(&self) -> bool {todo!()}
    
    pub(crate) fn pursue(&mut self,
                         this_person: PersonId,
                         this_module: &mut dyn ModulePersonInterface,
                         this_vessel: &dyn VesselPersonInterface,
    ) -> Result<ObjectiveStatus, CraftingModulesObjectiveError> {
        match self {
            Self::SearchingForCraftingModule { needed_capabilities,deploy } => {
                let is_recipe_set_suitable = |recipes: &[AssemblyRecipe],
                                              mut needed_caps: Vec<ModuleCapability>|
                                              -> bool {
                    for r in recipes {
                        for cap in r.output_capabilities() {
                            if let Some(i) = needed_caps.iter().position(|x| *x == *cap) {
                                needed_caps.remove(i);
                            }
                        }
                    }
                    needed_caps.is_empty()
                };

                if is_recipe_set_suitable(this_module.assembly_recipes(), needed_capabilities.clone()) {
                    *self = Self::MovingToCraftingModule {
                        dst: this_module.id(),
                        needed_capabilities: std::mem::take(needed_capabilities),
                        deploy: *deploy,
                    };
                    return Ok(ObjectiveStatus::InProgress)
                }

                for crafting_module in this_vessel.modules_with_cap(ModuleCapability::Crafting)
                {
                    if is_recipe_set_suitable(crafting_module.assembly_recipes(), needed_capabilities.clone()) {
                        *self = Self::MovingToCraftingModule {
                            dst: crafting_module.id(),
                            needed_capabilities: std::mem::take(needed_capabilities),
                            deploy: *deploy
                        };
                        return Ok(ObjectiveStatus::InProgress)
                    }
                }
                Err(CraftingModulesObjectiveError::CanNotFindCraftingModule)
            }
            Self::MovingToCraftingModule {
                dst,
                needed_capabilities,
                deploy,
            } => {
                if *dst == this_module.id() {
                    *self = Self::Crafting {
                        needed_capabilities: BTreeSet::from_iter(std::mem::take(needed_capabilities)),
                        deploy: *deploy,
                    };
                } else {
                    this_vessel.move_to_module(this_person, *dst);
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::Crafting {
                needed_capabilities,
                deploy,
            } => {
                if this_module.active_recipe().is_some() {
                    if !this_module.interact() {
                        todo!()
                    } else {
                        Ok(ObjectiveStatus::InProgress)
                    }
                } else if let Some(cap) = needed_capabilities.first() {
                    if let Some(recipe) = this_module.recipe_by_output_capability(*cap) {
                        if this_module.has_resources_for_recipe(recipe) {
                            let ok = this_module.start_assembly(recipe,*deploy);
                            assert!(ok);
                            for c in this_module.recipe_output_capabilities(recipe) {
                                needed_capabilities.remove(c);
                            }
                            Ok(ObjectiveStatus::InProgress)
                        } else {
                            todo!()
                        }
                    } else {
                        todo!()
                    }
                } else {
                    *self = Done;
                    Ok(ObjectiveStatus::Done)
                }
            }
            Done => Ok(ObjectiveStatus::Done),
        }
    }
}

#[derive(Debug)]
pub(crate) enum CraftingModulesObjectiveError {
    CanNotFindCraftingModule,
}

impl Display for CraftingModulesObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CraftingModulesObjectiveError {}