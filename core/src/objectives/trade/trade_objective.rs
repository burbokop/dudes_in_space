use crate::objectives::trade::{BuyGoodsObjective, SellGoodsObjective};
use dudes_in_space_api::environment::{
    EnvironmentContext, FindBestOfferPair, FindBestOfferPairResult, PlaceOrders, PlaceOrdersResult,
    RequestCreditLimitIncrease, RequestCreditLimitIncreaseResult,
};
use dudes_in_space_api::finance::{Money, WithdrawalError};
use dudes_in_space_api::item::ItemCount;
use dudes_in_space_api::module::{
    ModuleCapability, ModuleConsole, ModuleId, PlaceBuyOrderError, PlaceSellOrderError,
};
use dudes_in_space_api::person;
use dudes_in_space_api::person::{
    DynObjective, Objective, ObjectiveDecider, ObjectiveStatus, Passion, PersonLogger, ThisPerson,
    tie,
};
use dudes_in_space_api::utils::request::{ReqContext, ReqFuture, ReqFutureSeed, ReqTakeError};
use dudes_in_space_api::vessel::{VesselId, VesselInternalConsole};
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
#[deserialize_seed_xxx(seed = crate::objectives::trade::trade_objective::TradeObjectiveSeed::<'context>)]
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
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.find_future_seed)])]
    SearchForOffers {
        future: ReqFuture<FindBestOfferPairResult>,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.credit_limit_increase_future_seed)])]
    WaitForCreditLimitIncreased {
        future: ReqFuture<RequestCreditLimitIncreaseResult>,
        place_orders_request: PlaceOrders,
        total_money_needed: Money,
    },
    #[deserialize_seed_xxx(seeds = [(future, self.seed.seed.place_future_seed)])]
    WaitForOrdersToBePlaced {
        future: ReqFuture<PlaceOrdersResult>,
    },
    MoveToVesselToBuy {
        buy_goods_objective: BuyGoodsObjective,
    },
    MoveToVesselToSell {
        sell_goods_objective: SellGoodsObjective,
    },
}

#[derive(Clone)]
pub(crate) struct TradeObjectiveSeed<'context> {
    find_future_seed: ReqFutureSeed<'context, FindBestOfferPairResult>,
    place_future_seed: ReqFutureSeed<'context, PlaceOrdersResult>,
    credit_limit_increase_future_seed: ReqFutureSeed<'context, RequestCreditLimitIncreaseResult>,
}

impl<'context> TradeObjectiveSeed<'context> {
    pub(crate) fn new(context: &'context ReqContext) -> Self {
        Self {
            find_future_seed: ReqFutureSeed::new(context),
            place_future_seed: ReqFutureSeed::new(context),
            credit_limit_increase_future_seed: ReqFutureSeed::new(context),
        }
    }
}

impl TradeObjective {
    pub fn new() -> Self {
        Self::SearchVessel
    }
}

impl Objective for TradeObjective {
    type Result = ();
    type Error = TradeObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error> {
        match self {
            Self::SearchVessel => {
                if tie(this_module, this_vessel)
                    .has_primary_capabilities(NEEDED_PRIMARY_CAPABILITIES.iter().cloned())
                    && tie(this_module, this_vessel)
                        .has_capabilities(NEEDED_CAPABILITIES.iter().cloned())
                {
                    logger.info("SearchForCockpit");
                    *self = Self::SearchForCockpit;
                    return Ok(ObjectiveStatus::InProgress);
                }

                if let Some((vessel_id, docking_port_module_id)) = tie(this_module, this_vessel)
                    .for_each_docking_clamps_with_vessel_which_has_caps(
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
                        .move_person_to_docked_vessel(
                            environment_context.subordination_table(),
                            this_module,
                            *this_person.id,
                            connection_id,
                        )
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
                    *self = Self::SearchForOffers {
                        future: FindBestOfferPair {
                            free_storage_space: tie(this_module, this_vessel)
                                .total_primary_free_space(),
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
            Self::SearchForOffers { future } => match future.take() {
                Ok(search_result) => {
                    assert_ne!(search_result.max_profit_buy_offer.offer.count_range.end, 0);
                    assert_ne!(search_result.max_profit_sell_offer.offer.count_range.end, 0);
                    assert_eq!(
                        search_result.max_profit_buy_offer.offer.item,
                        search_result.max_profit_sell_offer.offer.item
                    );

                    let item = environment_context
                        .item_vault()
                        .get(search_result.max_profit_buy_offer.offer.item.clone())
                        .unwrap();
                    let item = item.upgrade().unwrap();

                    let count = ((tie(this_module, this_vessel).total_primary_free_space()
                        / item.volume) as ItemCount)
                        .min(search_result.max_profit_buy_offer.offer.count_range.end)
                        .min(search_result.max_profit_sell_offer.offer.count_range.end);

                    assert_ne!(count, 0);

                    let total_money_needed = search_result
                        .max_profit_buy_offer
                        .offer
                        .price_per_unit
                        .clone()
                        * count;

                    let place_orders_request = PlaceOrders {
                        customer_wallet: this_person.finance.wallet().id().clone(),
                        buy_offers: vec![(search_result.max_profit_buy_offer, count)],
                        sell_offers: vec![(search_result.max_profit_sell_offer, count)],
                    };

                    match this_person.ensure_has_money_in_wallet(
                        environment_context.bank_registry(),
                        environment_context.wallet_registry(),
                        total_money_needed.clone(),
                    ) {
                        Ok(_) => {}
                        Err(WithdrawalError::CreditLimitReached {
                            bank_owner,
                            requested,
                            limit,
                        }) => {
                            *self = Self::WaitForCreditLimitIncreased {
                                place_orders_request,
                                total_money_needed,
                                future: RequestCreditLimitIncrease {
                                    recipient: bank_owner,
                                    new_limit: requested,
                                }
                                .push(environment_context.request_storage_mut()),
                            };
                            return Ok(ObjectiveStatus::InProgress);
                        }
                        Err(WithdrawalError::InDebt) => todo!("person: {:?}", this_person),
                    }

                    *self = Self::WaitForOrdersToBePlaced {
                        future: place_orders_request
                            .push(environment_context.request_storage_mut()),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::WaitForCreditLimitIncreased {
                place_orders_request,
                future,
                total_money_needed,
            } => match future.take() {
                Ok(RequestCreditLimitIncreaseResult::LimitIncreased) => {
                    match this_person.ensure_has_money_in_wallet(
                        environment_context.bank_registry(),
                        environment_context.wallet_registry(),
                        total_money_needed.clone(),
                    ) {
                        Ok(_) => {}
                        Err(WithdrawalError::CreditLimitReached {
                            bank_owner,
                            requested,
                            limit,
                        }) => unreachable!(),
                        Err(WithdrawalError::InDebt) => todo!("person: {:?}", this_person),
                    }

                    *self = Self::WaitForOrdersToBePlaced {
                        future: place_orders_request
                            .clone()
                            .push(environment_context.request_storage_mut()),
                    };
                    Ok(ObjectiveStatus::InProgress)
                }
                Ok(RequestCreditLimitIncreaseResult::RequestDenied) => {
                    todo!()
                }
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::WaitForOrdersToBePlaced { future } => match future.take() {
                Ok(PlaceOrdersResult::Ok { .. }) => todo!(),
                Ok(PlaceOrdersResult::PlaceBuyOrderError(PlaceBuyOrderError::OfferNotFound)) => {
                    todo!()
                }
                Ok(PlaceOrdersResult::PlaceBuyOrderError(
                    PlaceBuyOrderError::CountIsNotInRange,
                )) => todo!(),
                Ok(PlaceOrdersResult::PlaceBuyOrderError(
                    PlaceBuyOrderError::NotEnoughMoneyInCustomerWallet,
                )) => unreachable!(
                    "Because `ensure_has_money_in_wallet` expected to be called before"
                ),
                Ok(PlaceOrdersResult::PlaceSellOrderError(PlaceSellOrderError::OfferNotFound)) => {
                    todo!()
                }
                Ok(PlaceOrdersResult::PlaceSellOrderError(
                    PlaceSellOrderError::CountIsNotInRange,
                )) => todo!(),
                Ok(PlaceOrdersResult::PlaceSellOrderError(
                    PlaceSellOrderError::EmptyOperationalWallet,
                )) => unreachable!(
                    "Because offers with no operational wallet are excluded from search"
                ),
                Ok(PlaceOrdersResult::PlaceSellOrderError(
                    PlaceSellOrderError::NotEnoughMoneyInOperationalWallet,
                )) => unreachable!(
                    "Because offers with that has not enough money in operational wallet are excluded from search"
                ),
                Err(ReqTakeError::Pending) => Ok(ObjectiveStatus::InProgress),
                Err(ReqTakeError::AlreadyTaken) => unreachable!(),
            },
            Self::MoveToVesselToBuy {
                buy_goods_objective,
            } => todo!(),
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
        person: &ThisPerson,
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

impl Display for TradeObjective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SearchVessel => write!(f, "SearchVessel"),
            Self::MoveToVessel { .. } => write!(f, "MoveToVessel"),
            Self::SearchForCockpit => write!(f, "SearchForCockpit"),
            Self::MoveToCockpit { .. } => write!(f, "MoveToCockpit"),
            Self::SearchForOffers { .. } => write!(f, "SearchForOffers"),
            Self::MoveToVesselToBuy { .. } => write!(f, "MoveToVesselToBuy"),
            Self::MoveToVesselToSell { .. } => write!(f, "MoveToVesselToSell"),
            Self::WaitForOrdersToBePlaced { .. } => write!(f, "WaitForOrdersToBePlaced"),
            Self::WaitForCreditLimitIncreased { .. } => write!(f, "WaitForCreditLimitIncreased"),
        }
    }
}
