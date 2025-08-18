use crate::item::{Item, ItemRefStack};
use crate::module::ModuleCapability;
use crate::person::Money;
use crate::vessel::VesselId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Weak;
use uuid::NonNilUuid;

#[derive(Debug, Serialize, Deserialize)]
struct BuyOrderImpl {
    vessel_to_buy_from: VesselId,
    items: Vec<ItemRefStack>,
    price: Money,
}

#[derive(Debug, Serialize, Deserialize)]
struct SellOrderImpl {
    vessel_to_sell_to: VesselId,
    items: Vec<ItemRefStack>,
    price: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakBuyOrder {
    id: NonNilUuid,
    data: Option<Weak<BuyOrderImpl>>,
}

impl WeakBuyOrder {
    pub fn vessel_to_buy_from(&self) -> Option<VesselId> {
        todo!()
    }
    pub fn items(&self) -> Option<Vec<Item>> {
        todo!()
    }
    pub fn price(&self) -> Option<Money> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyOrder {
    id: NonNilUuid,
    data: Rc<BuyOrderImpl>,
}

impl BuyOrder {
    pub fn new() -> (Self, WeakBuyOrder) {
        todo!()
    }

    pub fn vessel_to_buy_from(&self) -> VesselId {
        todo!()
    }
    pub fn items(&self) -> Vec<Item> {
        todo!()
    }
    pub fn price(&self) -> Money {
        todo!()
    }
}

pub struct WeakBuyVesselManualOrderEstimate {}

pub struct WeakBuyVesselOrder {}

impl WeakBuyVesselOrder {
    pub fn vessel_to_buy_from(&self) -> Option<VesselId> {
        todo!()
    }
    pub fn primary_caps(&self) -> Option<Vec<ModuleCapability>> {
        todo!()
    }
    pub fn price(&self) -> Option<Money> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakSellOrder {
    id: NonNilUuid,
    data: Option<Weak<SellOrderImpl>>,
}

impl WeakSellOrder {
    pub fn vessel_to_sell_to(&self) -> Option<VesselId> {
        todo!()
    }
    pub fn items(&self) -> Option<Vec<Item>> {
        todo!()
    }
    pub fn price(&self) -> Option<Money> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SellOrder {
    id: NonNilUuid,
    data: Rc<SellOrderImpl>,
}

impl SellOrder {
    pub fn new() -> (Self, WeakSellOrder) {
        todo!()
    }

    pub fn vessel_to_sell_to(&self) -> VesselId {
        todo!()
    }
    pub fn items(&self) -> Vec<Item> {
        todo!()
    }
    pub fn price(&self) -> Money {
        todo!()
    }
}

struct OrderHolder {
    buy_orders: BTreeMap<NonNilUuid, Weak<BuyOrderImpl>>,
    sell_orders: BTreeMap<NonNilUuid, Weak<WeakSellOrder>>,
}
