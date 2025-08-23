use crate::finance::MoneyRef;
use crate::item::{Item, ItemRefStack};
use crate::module::ModuleCapability;
use crate::utils::non_nil_uuid::NonNilUuid;
use crate::vessel::VesselId;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};

#[derive(Debug, Serialize, Deserialize)]
struct BuyOrderImpl {
    vessel_to_buy_from: VesselId,
    items: Vec<ItemRefStack>,
    price: MoneyRef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakBuyOrder {
    id: NonNilUuid,
    #[serde(skip)]
    data: Option<Weak<BuyOrderImpl>>,
}

impl WeakBuyOrder {
    pub fn vessel_to_buy_from(&self) -> Option<VesselId> {
        todo!()
    }
    pub fn items(&self) -> Option<Vec<Item>> {
        todo!()
    }
    pub fn price(&self) -> Option<MoneyRef> {
        todo!()
    }
}

#[derive(Debug)]
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
    pub fn price(&self) -> MoneyRef {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SellOrderImpl {
    vessel_to_sell_to: VesselId,
    items: Vec<ItemRefStack>,
    price: MoneyRef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakSellOrder {
    id: NonNilUuid,
    #[serde(skip)]
    data: Option<Weak<SellOrderImpl>>,
}

impl WeakSellOrder {
    pub fn vessel_to_sell_to(&self) -> Option<VesselId> {
        todo!()
    }
    pub fn items(&self) -> Option<Vec<Item>> {
        todo!()
    }
    pub fn price(&self) -> Option<MoneyRef> {
        todo!()
    }
}

#[derive(Debug)]
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
    pub fn price(&self) -> MoneyRef {
        todo!()
    }
}

pub struct WeakBuyCustomVesselOrderEstimate {}

impl WeakBuyCustomVesselOrderEstimate {
    pub fn money(&self) -> MoneyRef {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyVesselOrderImpl {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakBuyVesselOrder {
    id: NonNilUuid,
    #[serde(skip)]
    data: Option<Weak<BuyVesselOrderImpl>>,
}

impl WeakBuyVesselOrder {
    pub fn vessel_to_buy_from(&self) -> Option<VesselId> {
        todo!()
    }
    pub fn primary_caps(&self) -> Option<Vec<ModuleCapability>> {
        todo!()
    }
    pub fn price(&self) -> Option<MoneyRef> {
        todo!()
    }
}

#[derive(Debug)]
pub struct BuyVesselOrder {
    id: NonNilUuid,
    data: Rc<BuyVesselOrderImpl>,
}

pub struct OrderSeed<'h, T> {
    holder: &'h OrderHolder,
    _pd: std::marker::PhantomData<T>,
}

impl<'h, T> Clone for OrderSeed<'h, T> {
    fn clone(&self) -> Self {
        Self {
            holder: self.holder,
            _pd: Default::default(),
        }
    }
}

impl<'h, T> OrderSeed<'h, T> {
    pub fn new(holder: &'h OrderHolder) -> Self {
        Self {
            holder,
            _pd: Default::default(),
        }
    }
}

impl Serialize for BuyOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Impl<'a> {
            data: &'a BuyOrderImpl,
            id: NonNilUuid,
        }

        Impl {
            data: &self.data,
            id: self.id,
        }
        .serialize(serializer)
    }
}

impl<'de, 'context> DeserializeSeed<'de> for OrderSeed<'context, BuyOrder> {
    type Value = BuyOrder;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Impl {
            data: BuyOrderImpl,
            id: NonNilUuid,
        }

        let Impl { data, id } = Impl::deserialize(deserializer)?;
        Ok(self.holder.register_buy_order(data, id))
    }
}

impl Serialize for SellOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Impl<'a> {
            data: &'a SellOrderImpl,
            id: NonNilUuid,
        }

        Impl {
            data: &self.data,
            id: self.id,
        }
        .serialize(serializer)
    }
}

impl<'de, 'context> DeserializeSeed<'de> for OrderSeed<'context, SellOrder> {
    type Value = SellOrder;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Impl {
            data: SellOrderImpl,
            id: NonNilUuid,
        }

        let Impl { data, id } = Impl::deserialize(deserializer)?;
        Ok(self.holder.register_sell_order(data, id))
    }
}

impl Serialize for BuyVesselOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Impl<'a> {
            data: &'a BuyVesselOrderImpl,
            id: NonNilUuid,
        }

        Impl {
            data: &self.data,
            id: self.id,
        }
        .serialize(serializer)
    }
}

impl<'de, 'context> DeserializeSeed<'de> for OrderSeed<'context, BuyVesselOrder> {
    type Value = BuyVesselOrder;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Impl {
            data: BuyVesselOrderImpl,
            id: NonNilUuid,
        }

        let Impl { data, id } = Impl::deserialize(deserializer)?;
        Ok(self.holder.register_buy_vessel_order(data, id))
    }
}

impl BuyVesselOrder {
    pub fn new() -> (Self, WeakBuyVesselOrder) {
        todo!()
    }

    pub fn primary_caps(&self) -> BTreeSet<ModuleCapability> {
        todo!()
    }

    pub fn price(&self) -> MoneyRef {
        todo!()
    }
}

pub struct OrderHolder {
    buy_orders: RefCell<BTreeMap<NonNilUuid, Weak<BuyOrderImpl>>>,
    sell_orders: RefCell<BTreeMap<NonNilUuid, Weak<SellOrderImpl>>>,
    buy_vessel_orders: RefCell<BTreeMap<NonNilUuid, Weak<BuyVesselOrderImpl>>>,
}

impl OrderHolder {
    pub fn new() -> Self {
        Self {
            buy_orders: RefCell::new(Default::default()),
            sell_orders: RefCell::new(Default::default()),
            buy_vessel_orders: RefCell::new(Default::default()),
        }
    }

    fn register_buy_order(&self, data: BuyOrderImpl, id: NonNilUuid) -> BuyOrder {
        let data = Rc::new(data);
        self.buy_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        BuyOrder { data, id }
    }

    fn register_sell_order(&self, data: SellOrderImpl, id: NonNilUuid) -> SellOrder {
        let data = Rc::new(data);
        self.sell_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        SellOrder { data, id }
    }

    fn register_buy_vessel_order(
        &self,
        data: BuyVesselOrderImpl,
        id: NonNilUuid,
    ) -> BuyVesselOrder {
        let data = Rc::new(data);
        self.buy_vessel_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        BuyVesselOrder { data, id }
    }
}
