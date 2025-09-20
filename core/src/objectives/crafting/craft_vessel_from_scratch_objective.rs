use crate::objectives::crafting::{
    BuildVesselObjective, BuildVesselObjectiveError, CraftModulesObjective,
    CraftModulesObjectiveError, CraftModulesObjectiveOptions,
};
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{Bank, Money};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleStorage};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::utils::math::Zero;
use dudes_in_space_api::vessel::VesselInternalConsole;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "craft_vessel_from_scratch_objective_stage")]
pub(crate) enum CraftVesselFromScratchObjective {
    CheckingAllPrerequisites {
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        wait_if_has_no_ingredients: bool,
        total_cost_price: Money,
    },
    CraftingDockyard {
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        wait_if_has_no_ingredients: bool,
        crafting_objective: CraftModulesObjective,
        total_cost_price: Money,
    },
    CraftingVesselModules {
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        wait_if_has_no_ingredients: bool,
        crafting_objective: CraftModulesObjective,
        total_cost_price: Money,
    },
    BuildingVessel {
        needed_capabilities: Vec<ModuleCapability>,
        needed_primary_capabilities: Vec<ModuleCapability>,
        wait_if_has_no_ingredients: bool,
        building_objective: BuildVesselObjective,
        total_cost_price: Money,
    },
    Done {
        result: CraftVesselFromScratchObjectiveResult,
    },
}

struct DockyardRef<'x> {
    module_storages: &'x [ModuleStorage],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CraftVesselFromScratchObjectiveResult {
    pub cost_price: Money,
}

impl CraftVesselFromScratchObjective {
    pub(crate) fn new(
        needed_capabilities: BTreeSet<ModuleCapability>,
        needed_primary_capabilities: BTreeSet<ModuleCapability>,
        wait_if_has_no_ingredients: bool,
        this_person: &mut ThisPerson,
        environment_context: &EnvironmentContext,
    ) -> Self {
        let this_person_wallet_id = this_person.finance.wallet().id().clone();
        Self::CheckingAllPrerequisites {
            needed_capabilities: needed_capabilities.into_iter().collect(),
            needed_primary_capabilities: needed_primary_capabilities.into_iter().collect(),
            wait_if_has_no_ingredients,
            total_cost_price: Money {
                currency: this_person.finance.preferred_currency_or_create(
                    environment_context.bank_registry(),
                    Bank::new(
                        this_person.id.clone(),
                        this_person_wallet_id,
                        environment_context.currency_generator().generate_name(
                            &mut rng(),
                            environment_context.bank_registry(),
                            this_person,
                        ),
                    ),
                ),
                amount: Zero::zero(),
            },
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
    type Result = CraftVesselFromScratchObjectiveResult;
    type Error = CraftVesselFromScratchObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::CheckingAllPrerequisites {
                needed_capabilities,
                needed_primary_capabilities,
                wait_if_has_no_ingredients,
                total_cost_price,
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
                        needed_capabilities: std::mem::take(needed_capabilities),
                        needed_primary_capabilities: std::mem::take(needed_primary_capabilities),
                        wait_if_has_no_ingredients: std::mem::take(wait_if_has_no_ingredients),
                        crafting_objective: CraftModulesObjective::new(
                            BTreeSet::from([ModuleCapability::Dockyard]),
                            BTreeSet::from([]),
                            Default::default(),
                            logger,
                        ),
                        total_cost_price: total_cost_price.clone(),
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
                        needed_capabilities: needed_capabilities.clone(),
                        needed_primary_capabilities: needed_primary_capabilities.clone(),
                        wait_if_has_no_ingredients: std::mem::take(wait_if_has_no_ingredients),
                        crafting_objective: CraftModulesObjective::new(
                            std::mem::take(needed_capabilities).into_iter().collect(),
                            std::mem::take(needed_primary_capabilities)
                                .into_iter()
                                .collect(),
                            CraftModulesObjectiveOptions {
                                deploy: false,
                                wait_if_has_no_ingredients: *wait_if_has_no_ingredients,
                            },
                            logger,
                        ),
                        total_cost_price: total_cost_price.clone(),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                logger.info("Beginning vessel building stage...");
                *self = Self::BuildingVessel {
                    needed_capabilities: needed_capabilities.clone(),
                    needed_primary_capabilities: needed_primary_capabilities.clone(),
                    wait_if_has_no_ingredients: std::mem::take(wait_if_has_no_ingredients),
                    building_objective: BuildVesselObjective::new(
                        std::mem::take(needed_capabilities),
                        std::mem::take(needed_primary_capabilities),
                    ),
                    total_cost_price: total_cost_price.clone(),
                };
                Ok(ObjectiveStatus::InProgress)
            }
            Self::CraftingDockyard {
                needed_capabilities,
                needed_primary_capabilities,
                wait_if_has_no_ingredients,
                crafting_objective,
                total_cost_price,
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
                    ObjectiveStatus::Done(result) => {
                        total_cost_price
                            .add_assign_same_currency(result.cost_price)
                            .unwrap();

                        logger.info("CraftVesselFromScratchObjective::CraftingDockyard::CheckingAllPrerequisites");
                        *self = Self::CheckingAllPrerequisites {
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                            wait_if_has_no_ingredients: std::mem::take(wait_if_has_no_ingredients),
                            total_cost_price: total_cost_price.clone(),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::CraftingVesselModules {
                needed_capabilities,
                needed_primary_capabilities,
                crafting_objective,
                wait_if_has_no_ingredients,
                total_cost_price,
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
                    ObjectiveStatus::Done(result) => {
                        total_cost_price
                            .add_assign_same_currency(result.cost_price)
                            .unwrap();
                        logger.info(
                            "Checking all prerequisites for crafting a vessel from scratch...",
                        );
                        *self = Self::CheckingAllPrerequisites {
                            needed_capabilities: std::mem::take(needed_capabilities),
                            needed_primary_capabilities: std::mem::take(
                                needed_primary_capabilities,
                            ),
                            wait_if_has_no_ingredients: std::mem::take(wait_if_has_no_ingredients),
                            total_cost_price: total_cost_price.clone(),
                        }
                    }
                }
                Ok(ObjectiveStatus::InProgress)
            }
            Self::BuildingVessel {
                needed_capabilities,
                needed_primary_capabilities,
                wait_if_has_no_ingredients,
                building_objective,
                total_cost_price,
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
                    ObjectiveStatus::Done(_) => {
                        logger.info("Done crafting a vessel from scratch.");

                        let result: CraftVesselFromScratchObjectiveResult = (|| todo!())();

                        *self = Self::Done {
                            result: result.clone(),
                        };
                        Ok(ObjectiveStatus::Done(result))
                    }
                }
            }
            Self::Done { result } => Ok(ObjectiveStatus::Done(result.clone())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
