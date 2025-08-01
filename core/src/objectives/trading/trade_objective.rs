use dudes_in_space_api::module::{ModuleCapability, ModuleConsole, ModuleId, ProcessTokenContext};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{
    Awareness, Boldness, DynObjective, Gender, Morale, Objective, ObjectiveDecider,
    ObjectiveStatus, Passion, PersonId, PersonLogger,
};
use dudes_in_space_api::vessel::{VesselConsole, VesselId};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::ControlFlow;
use crate::objectives::trading::{BuyGoodsObjective, SellGoodsObjective};

static TYPE_ID: &str = "TradeObjective";

static NEEDED_PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::ItemStorage];

static NEEDED_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum TradeObjective {
    SearchVessel,
    MoveToVessel { vessel_id: VesselId, docking_port_module_id: Option<ModuleId> },
    SearchForCockpit,
    MoveToCockpit,
    SearchForBuyOffers,
    MoveToVesselToBuy {
        buy_goods_objective: BuyGoodsObjective
    },
    SearchForSellOffers,
    MoveToVesselToSell {
        sell_goods_objective: SellGoodsObjective
    },
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
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
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
                            ControlFlow::Break((
                                entry.clamp.vessel_docked().unwrap().id(),
                                entry.module.map(|x| x.id()),
                            ))
                        },
                    )
                    .break_value()
                {
                    logger.info("Moving to vessel...");
                    *self = Self::MoveToVessel { vessel_id, docking_port_module_id };
                    return Ok(ObjectiveStatus::InProgress);
                }

                Err(TradeObjectiveError::SuitableVesselNotFound)
            }
            Self::MoveToVessel { vessel_id, docking_port_module_id } => {
                match docking_port_module_id {
                    None => {
                         let vessel = person::utils::find_docking_clamp_with_vessel_with_id_mut(this_module.docking_clamps_mut(), *vessel_id)
                             .unwrap()
                             .vessel_docked_mut()
                             .unwrap();

                        if let Some(docking_connector) = vessel.modules_with_capability_mut(ModuleCapability::DockingConnector).next() {
                            // cockpit
                        } else {
                            
                        }
                        
                        todo!()
                    },
                    Some(_) => todo!(),
                }
            },
            Self::SearchForCockpit => todo!(),
            Self::MoveToCockpit => todo!(),
            Self::SearchForBuyOffers => todo!(),
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
        person_id: PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        if passions.contains(&Passion::Trade) || passions.contains(&Passion::Money) {
            Some(Box::new(TradeObjective::new()))
        } else {
            None
        }
    }
}

pub(crate) struct TradeObjectiveDynSeed;

impl DynDeserializeSeed<dyn DynObjective> for TradeObjectiveDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Result<Box<dyn DynObjective>, Box<dyn Error>> {
        let obj: TradeObjective = from_intermediate(&intermediate).map_err(Box::new)?;
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
