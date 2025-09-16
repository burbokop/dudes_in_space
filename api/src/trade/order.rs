use crate::finance::{Money, Wallet, WalletId};
use crate::item::{Item, ItemRefStack};
use crate::module::{
    ModuleCapability, ProcessToken, ProcessTokenContext, ProcessTokenMut, ProcessTokenMutSeed,
};
use crate::utils::non_nil_uuid::NonNilUuid;
use crate::vessel::VesselId;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};

#[derive(Debug, Serialize, Deserialize)]
struct BuyOrderImpl {
    vessel_to_buy_from: VesselId,
    items: Vec<ItemRefStack>,
    price: Money,
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
    pub fn price(&self) -> Option<Money> {
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
    pub fn price(&self) -> Money {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SellOrderImpl {
    vessel_to_sell_to: VesselId,
    items: Vec<ItemRefStack>,
    price: Money,
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
    pub fn price(&self) -> Option<Money> {
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
    pub fn price(&self) -> Money {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyCustomVesselOrderEstimate {
    pub estimate: Money,
    pub pledge: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyVesselOrderImpl {
    count: usize,
}

// TODO: must transfer money from pledge wallet back to customer wallet if order if canceled (or dropped, for example if module containing the order is destroyed).
// Possible implementation can be by saving pledge wallet to wallet registry in special map for pledge wallets which contains also id of customer wallet
// and on `Environment::proceed` all pledge wallets with ony one ref will be dropped and their money transferred to corresponding customer wallets.
// In this case pledge wallets that are destroyed on app exit won't be considered as "lost" because `Environment::proceed` shouldn't be called after serialization.
#[derive(Debug, Serialize, Deserialize)]
pub struct BuyCustomVesselOrderImpl {
    pledge_wallet: Wallet,
    customer_wallet_id: WalletId,
    capabilities: BTreeSet<ModuleCapability>,
    primary_capabilities: BTreeSet<ModuleCapability>,
    count: usize,
}

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
    pub fn price(&self) -> Option<Money> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakBuyCustomVesselOrder {
    id: NonNilUuid,
    #[serde(skip)]
    data: Option<Weak<BuyCustomVesselOrderImpl>>,
    process_token: ProcessToken,
}

impl WeakBuyCustomVesselOrder {
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

#[derive(Debug)]
pub struct BuyVesselOrder {
    id: NonNilUuid,
    data: Rc<BuyVesselOrderImpl>,
}

#[derive(Debug)]
pub struct BuyCustomVesselOrder {
    id: NonNilUuid,
    data: Rc<BuyCustomVesselOrderImpl>,
    process_token: ProcessTokenMut,
}

pub struct OrderSeed<'h, T> {
    holder: &'h OrderHolder,
    _pd: std::marker::PhantomData<T>,
}

#[derive(Clone)]
pub struct BuyCustomVesselOrderSeed<'h, 'c> {
    holder: &'h OrderHolder,
    process_token_context: &'c ProcessTokenContext,
}

impl<'h, 'c> BuyCustomVesselOrderSeed<'h, 'c> {
    pub fn new(holder: &'h OrderHolder, process_token_context: &'c ProcessTokenContext) -> Self {
        Self {
            holder,
            process_token_context,
        }
    }
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

impl Serialize for BuyCustomVesselOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Impl<'a> {
            data: &'a BuyCustomVesselOrderImpl,
            id: NonNilUuid,
        }

        Impl {
            data: &self.data,
            id: self.id,
        }
        .serialize(serializer)
    }
}

impl<'de, 'holder, 'context> DeserializeSeed<'de> for BuyCustomVesselOrderSeed<'holder, 'context> {
    type Value = BuyCustomVesselOrder;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(DeserializeSeedXXX)]
        #[deserialize_seed_xxx(seed = ImplSeed::<'context>)]
        struct Impl {
            id: NonNilUuid,
            data: BuyCustomVesselOrderImpl,
            #[deserialize_seed_xxx(seed = self.seed.process_token_seed)]
            process_token: ProcessTokenMut,
        }

        #[derive(Clone)]
        struct ImplSeed<'context> {
            process_token_seed: ProcessTokenMutSeed<'context>,
        }

        let seed = ImplSeed {
            process_token_seed: ProcessTokenMutSeed::new(self.process_token_context),
        };

        let Impl {
            data,
            id,
            process_token,
        } = seed.deserialize(deserializer)?;
        Ok(self
            .holder
            .register_buy_custom_vessel_order(id, data, process_token))
    }
}

impl BuyCustomVesselOrder {
    pub fn new(
        pledge_wallet: Wallet,
        customer_wallet_id: WalletId,
        capabilities: BTreeSet<ModuleCapability>,
        primary_capabilities: BTreeSet<ModuleCapability>,
        count: usize,
    ) -> (WeakBuyCustomVesselOrder, Self) {
        let (process_token, process_token_mut) = ProcessTokenMut::new();

        let data = Rc::new(BuyCustomVesselOrderImpl {
            pledge_wallet,
            customer_wallet_id,
            capabilities,
            primary_capabilities,
            count,
        });

        let id = NonNilUuid::new_v4();
        (
            WeakBuyCustomVesselOrder {
                id,
                data: Some(Rc::downgrade(&data)),
                process_token,
            },
            Self {
                id,
                data,
                process_token: process_token_mut,
            },
        )
    }

    pub fn capabilities(&self) -> &BTreeSet<ModuleCapability> {
        &self.data.capabilities
    }

    pub fn primary_capabilities(&self) -> &BTreeSet<ModuleCapability> {
        &self.data.primary_capabilities
    }

    pub fn price(&self) -> Money {
        todo!()
    }

    pub fn pledge_wallet(&self) -> &Wallet {
        &self.data.pledge_wallet
    }

    pub fn pledge_wallet_mut(&mut self) -> &mut Wallet {
        todo!()
        // &mut self.data.pledge_wallet
    }
}

pub struct OrderHolder {
    buy_orders: RefCell<BTreeMap<NonNilUuid, Weak<BuyOrderImpl>>>,
    sell_orders: RefCell<BTreeMap<NonNilUuid, Weak<SellOrderImpl>>>,
    buy_vessel_orders: RefCell<BTreeMap<NonNilUuid, Weak<BuyVesselOrderImpl>>>,
    buy_custom_vessel_orders: RefCell<BTreeMap<NonNilUuid, Weak<BuyCustomVesselOrderImpl>>>,
}

impl OrderHolder {
    pub fn new() -> Self {
        Self {
            buy_orders: RefCell::new(Default::default()),
            sell_orders: RefCell::new(Default::default()),
            buy_vessel_orders: RefCell::new(Default::default()),
            buy_custom_vessel_orders: RefCell::new(Default::default()),
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

    fn register_buy_custom_vessel_order(
        &self,
        id: NonNilUuid,
        data: BuyCustomVesselOrderImpl,
        process_token: ProcessTokenMut,
    ) -> BuyCustomVesselOrder {
        let data = Rc::new(data);
        self.buy_custom_vessel_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        BuyCustomVesselOrder {
            id,
            data,
            process_token,
        }
    }
}
