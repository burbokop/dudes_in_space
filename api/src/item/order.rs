use crate::item::Item;
use crate::vessel::VesselId;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Weak;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type Money = usize;

#[derive(Debug, Serialize, Deserialize)]
struct BuyOrderImpl {
    vessel_to_buy_from: VesselId,
    items: Vec<Item>,
    price: Money,
}

#[derive(Debug, Serialize, Deserialize)]
struct SellOrderImpl {
    vessel_to_sell_to: VesselId,
    items: Vec<Item>,
    price: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakBuyOrder {
    id: Uuid,
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
    id: Uuid,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakSellOrder {
    id: Uuid,
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
    id: Uuid,
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
    buy_orders: BTreeMap<Uuid, Weak<BuyOrderImpl>>,
    sell_orders: BTreeMap<Uuid, Weak<WeakSellOrder>>,
}
