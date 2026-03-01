use crate::finance::{Money, Wallet, WalletId, WalletRegistry};
use crate::item::{Item, ItemCount, ItemId, ItemRefStack};
use crate::module::{
    ModuleCapability, ProcessToken, ProcessTokenContext, ProcessTokenExpiredError, ProcessTokenMut,
    ProcessTokenMutSeed,
};
use crate::utils::non_nil_uuid::NonNilUuid;
use crate::vessel::VesselId;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};

pub type OrderId = NonNilUuid;

#[derive(Debug, Serialize, Deserialize)]
struct BuyOrderImpl {
    vessel_to_buy_from: VesselId,
    items: Vec<ItemRefStack>,
    pledge_wallet: Rc<RefCell<Wallet>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakBuyOrder {
    id: OrderId,
    /// None if is not yet loaded from OrderHolder. Should be loaded lazily when needed
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
    id: OrderId,
    data: Rc<BuyOrderImpl>,
}

// TODO: must transfer money from pledge wallet back to customer wallet if order if canceled (or dropped, for example if module containing the order is destroyed).
// Possible implementation can be by saving pledge wallet to wallet registry in special map for pledge wallets which contains also id of customer wallet
// and on `Environment::proceed` all pledge wallets with ony one ref will be dropped and their money transferred to corresponding customer wallets.
// In this case pledge wallets that are destroyed on app exit won't be considered as "lost" because `Environment::proceed` shouldn't be called after serialization.
impl BuyOrder {
    pub fn new(
        wallet_registry: &WalletRegistry,
        pledge_wallet: Wallet,
        customer_wallet_id: WalletId,
        vessel_to_buy_from: VesselId,
        item: ItemId,
        count: ItemCount,
    ) -> (Self, WeakBuyOrder) {
        let id = OrderId::new_v4();

        let pledge_wallet = wallet_registry
            .register_pledge_wallet(pledge_wallet, customer_wallet_id)
            .unwrap();

        let data = Rc::new(BuyOrderImpl {
            vessel_to_buy_from,
            items: vec![ItemRefStack { id: item, count }],
            pledge_wallet,
        });

        let weak = WeakBuyOrder {
            id,
            data: Some(Rc::downgrade(&data)),
        };

        (Self { id, data }, weak)
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
    pledge_wallet: Rc<RefCell<Wallet>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeakSellOrder {
    id: OrderId,
    /// None if is not yet loaded from OrderHolder. Should be loaded lazily when needed
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

// TODO: must transfer money from pledge wallet back to owner wallet if order if canceled (or dropped, for example if module containing the order is destroyed).
// Possible implementation can be by saving pledge wallet to wallet registry in special map for pledge wallets which contains also id of customer wallet
// and on `Environment::proceed` all pledge wallets with ony one ref will be dropped and their money transferred to corresponding customer wallets.
// In this case pledge wallets that are destroyed on app exit won't be considered as "lost" because `Environment::proceed` shouldn't be called after serialization.
#[derive(Debug)]
pub struct SellOrder {
    id: OrderId,
    data: Rc<SellOrderImpl>,
}

impl SellOrder {
    pub fn new(
        wallet_registry: &WalletRegistry,
        pledge_wallet: Wallet,
        owner_wallet_id: WalletId,
        vessel_to_sell_to: VesselId,
        item: ItemId,
        count: ItemCount,
    ) -> (Self, WeakSellOrder) {
        let id = OrderId::new_v4();

        let pledge_wallet = wallet_registry
            .register_pledge_wallet(pledge_wallet, owner_wallet_id)
            .unwrap();

        let data = Rc::new(SellOrderImpl {
            vessel_to_sell_to,
            items: vec![ItemRefStack { id: item, count }],
            pledge_wallet,
        });

        let weak = WeakSellOrder {
            id,
            data: Some(Rc::downgrade(&data)),
        };

        (Self { id, data }, weak)
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
    id: OrderId,
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
    id: OrderId,
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

    /// If the order is completed, it means that all vessels are built and waiting to be picked up
    pub fn is_completed(
        &mut self,
        context: &ProcessTokenContext,
    ) -> Result<bool, ProcessTokenExpiredError> {
        self.process_token.is_completed(context)
    }
}

#[derive(Debug)]
pub struct BuyVesselOrder {
    id: OrderId,
    data: Rc<BuyVesselOrderImpl>,
}

#[derive(Debug)]
pub struct BuyCustomVesselOrder {
    id: OrderId,
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
            id: OrderId,
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
            id: OrderId,
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
            id: OrderId,
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
            id: OrderId,
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
            id: OrderId,
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
            id: OrderId,
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
            id: OrderId,
            data: &'a BuyCustomVesselOrderImpl,
            process_token: &'a ProcessTokenMut,
        }

        Impl {
            id: self.id,
            data: &self.data,
            process_token: &self.process_token,
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
            id: OrderId,
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

        let id = OrderId::new_v4();
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

    pub fn id(&self) -> &OrderId {
        &self.id
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
    buy_orders: RefCell<BTreeMap<OrderId, Weak<BuyOrderImpl>>>,
    sell_orders: RefCell<BTreeMap<OrderId, Weak<SellOrderImpl>>>,
    buy_vessel_orders: RefCell<BTreeMap<OrderId, Weak<BuyVesselOrderImpl>>>,
    buy_custom_vessel_orders: RefCell<BTreeMap<OrderId, Weak<BuyCustomVesselOrderImpl>>>,
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

    fn register_buy_order(&self, data: BuyOrderImpl, id: OrderId) -> BuyOrder {
        let data = Rc::new(data);
        self.buy_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        BuyOrder { data, id }
    }

    fn register_sell_order(&self, data: SellOrderImpl, id: OrderId) -> SellOrder {
        let data = Rc::new(data);
        self.sell_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        SellOrder { data, id }
    }

    fn register_buy_vessel_order(&self, data: BuyVesselOrderImpl, id: OrderId) -> BuyVesselOrder {
        let data = Rc::new(data);
        self.buy_vessel_orders
            .borrow_mut()
            .try_insert(id, Rc::downgrade(&data))
            .unwrap();
        BuyVesselOrder { data, id }
    }

    fn register_buy_custom_vessel_order(
        &self,
        id: OrderId,
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
