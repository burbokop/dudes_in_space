use crate::objectives::trading::{BuyGoodsObjective, SellGoodsObjective};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestBuyOffer, FindBestBuyOfferResult,
};
use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonInfo, PersonLogger,
};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::{VesselConsole, VesselId};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::ControlFlow;
use std::rc::Rc;

static TYPE_ID: &str = "TradeObjective";

static NEEDED_PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];

static NEEDED_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ModulePtr {
    This,
    Other(ModuleId),
}

impl From<Option<ModuleId>> for ModulePtr {
    fn from(value: Option<ModuleId>) -> Self {
        match value {
            None => Self::This,
            Some(v) => Self::Other(v),
        }
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "trade_objective_stage")]
#[deserialize_seed_xxx(seed = crate::objectives::trading::trade_objective::TradeObjectiveSeed::<'context>)]
pub(crate) enum TradeObjective {
    SearchVessel,
    MoveToVessel {
        vessel_id: VesselId,
        docking_port_module_id: ModulePtr,
    },
    SearchForCockpit,
    MoveToCockpit {
        dst: ModuleId,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.req_future_seed)])]
    SearchForBuyOffers {
        future: ReqFuture<FindBestBuyOfferResult>,
    },
    MoveToVesselToBuy {
        buy_goods_objective: BuyGoodsObjective,
    },
    SearchForSellOffers,
    MoveToVesselToSell {
        sell_goods_objective: SellGoodsObjective,
    },
}

#[derive(Clone)]
pub(crate) struct TradeObjectiveSeed<'context> {
    req_future_seed: ReqFutureSeed<'context, FindBestBuyOfferResult>,
}

impl<'context> TradeObjectiveSeed<'context> {
    pub fn new(context: &'context ReqContext) -> Self {
        Self {
            req_future_seed: ReqFutureSeed::new(context),
        }
    }
}

impl TradeObjective {
    pub fn new() -> Self {
        Self::SearchVessel
    }
}

impl Objective for TradeObjective {
    type Error = TradeObjectiveError;

    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        match self {
            Self::SearchVessel => {
                if person::utils::this_vessel_has_primary_caps(
                    this_module,
                    this_vessel,
                    NEEDED_PRIMARY_CAPABILITIES.iter().cloned(),
                ) && person::utils::this_vessel_has_caps(
                    this_module,
                    this_vessel,
                    NEEDED_CAPABILITIES.iter().cloned(),
                ) {
                    logger.info("SearchForCockpit");
                    *self = Self::SearchForCockpit;
                    return Ok(ObjectiveStatus::InProgress);
                }

                if let Some((vessel_id, docking_port_module_id)) =
                    person::utils::for_each_docking_clamps_with_vessel_which_has_caps(
                        this_module,
                        this_vessel,
                        NEEDED_CAPABILITIES,
                        NEEDED_PRIMARY_CAPABILITIES,
                        |entry| {
                            if entry
                                .clamp
                                .connection()
                                .unwrap()
                                .vessel
                                .modules_with_capability(ModuleCapability::PersonnelRoom)
                                .map(|m| m.free_person_slots_count())
                                .sum::<usize>()
                                > 0
                            {
                                ControlFlow::Break((
                                    entry.clamp.connection().unwrap().vessel.id(),
                                    entry.module.map(|x| x.id()),
                                ))
                            } else {
                                ControlFlow::Continue(())
                            }
                        },
                    )
                    .break_value()
                {
                    logger.info("Moving to vessel...");
                    *self = Self::MoveToVessel {
                        vessel_id,
                        docking_port_module_id: docking_port_module_id.into(),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                Err(TradeObjectiveError::SuitableVesselNotFound)
            }
            Self::MoveToVessel {
                vessel_id,
                docking_port_module_id,
            } => match docking_port_module_id {
                ModulePtr::This => {
                    let connection_id = person::utils::find_docking_clamp_with_vessel_with_id_mut(
                        this_module.docking_clamps_mut(),
                        *vessel_id,
                    )
                    .unwrap()
                    .connection()
                    .unwrap()
                    .connector_id;

                    this_vessel
                        .move_person_to_docked_vessel(this_module, *this_person.id, connection_id)
                        .unwrap();

                    *self = Self::SearchForCockpit;
                    Ok(ObjectiveStatus::InProgress)
                }
                ModulePtr::Other(_) => todo!("Move to vessel with docking port"),
            },
            Self::SearchForCockpit => {
                if this_module
                    .capabilities()
                    .contains(&ModuleCapability::Cockpit)
                {
                    logger.info("Already in a cockpit.");
                    *self = Self::SearchForBuyOffers {
                        future: FindBestBuyOffer {
                            free_storage_space: person::utils::total_primary_free_space(
                                this_module,
                                this_vessel,
                            ),
                        }
                        .push(environment_context.request_storage_mut()),
                    };
                    return Ok(ObjectiveStatus::InProgress);
                }

                for module in this_vessel.modules_with_capability(ModuleCapability::Cockpit) {
                    if module.free_person_slots_count() > 0 {
                        logger.info("Moving to dockyard module...");
                        *self = Self::MoveToCockpit { dst: module.id() };
                        return Ok(ObjectiveStatus::InProgress);
                    }
                }
                todo!()
            }
            Self::MoveToCockpit { dst } => todo!(),
            Self::SearchForBuyOffers { future } => match future.take() {
                Ok(x) => todo!(),
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::MoveToVesselToBuy { .. } => todo!(),
            Self::SearchForSellOffers => todo!(),
            Self::MoveToVesselToSell { .. } => todo!(),
        }
    }
}

impl DynSerialize for TradeObjective {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct TradeObjectiveDecider;

impl ObjectiveDecider for TradeObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if person.passions.contains(&Passion::Trade) || person.passions.contains(&Passion::Money) {
            Some(Box::new(TradeObjective::new()))
        } else {
            None
        }
    }
}

pub(crate) struct TradeObjectiveDynSeed {
    req_context: Rc<ReqContext>,
}

impl TradeObjectiveDynSeed {
    pub(crate) fn new(req_context: Rc<ReqContext>) -> Self {
        Self { req_context }
    }
}

impl DynDeserializeSeed<dyn DynObjective> for TradeObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let obj: TradeObjective =
            from_intermediate_seed(TradeObjectiveSeed::new(&self.req_context), &intermediate)
                .map_err(Box::new)?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug)]
pub(crate) enum TradeObjectiveError {
    SuitableVesselNotFound,
}

impl Display for TradeObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TradeObjectiveError {}
