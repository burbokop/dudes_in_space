use crate::CORE_PACKAGE_ID;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{
    BankRegistry, Money, NotEnoughMoneyInWallet, Wallet, WalletId, WalletRegistry,
};
use dudes_in_space_api::item::{ItemCount, ItemId, ItemSafe, ItemStorage, ItemVault, StorageRole};
use dudes_in_space_api::module::{
    AdminTradingConsole, CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole,
    ModuleId, ModuleStorage, ModuleTypeId, PackageId, PlaceBuyOrderError, PlaceSellOrderError,
    TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::trade::{
    BuyCustomVesselOffer, BuyCustomVesselOrder, BuyCustomVesselOrderEstimate, BuyOffer, BuyOrder,
    BuyVesselOffer, BuyVesselOrder, OfferId, OrderHolder, OrderSeed, SellOffer, SellOrder,
    WeakBuyCustomVesselOrder, WeakBuyOrder, WeakBuyVesselOrder, WeakSellOrder,
};
use dudes_in_space_api::utils::range::RangeInclusive;
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselId, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, VecSeed,
    from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Deserializer, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

pub(super) static TYPE_ID: &str = "TradingTerminal";
static FACTORY_TYPE_ID: &str = "TradingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::TradingTerminal,
    ModuleCapability::PersonnelRoom,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::TradingTerminal];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::trading_terminal::TradingTerminalSeed::<'h,'a,'b,'v>)]
pub(crate) struct TradingTerminal {
    id: ModuleId,
    buy_offers: Vec<BuyOffer>,
    sell_offers: Vec<SellOffer>,
    #[deserialize_seed_xxx(seed = self.seed.buy_order_seed)]
    buy_orders: Vec<BuyOrder>,
    #[deserialize_seed_xxx(seed = self.seed.sell_order_seed)]
    sell_orders: Vec<SellOrder>,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
    #[serde(default, deserialize_with = "deserialize_operational_wallet")]
    operational_wallet: Option<WalletId>,
}

fn deserialize_operational_wallet<'de, D>(data: D) -> Result<Option<WalletId>, D::Error>
where
    D: Deserializer<'de>,
{
    let x = WalletId::deserialize(data).map(Some).unwrap_or(None);
    Ok(x)
}

struct TradingTerminalSeed<'h, 'a, 'b, 'v> {
    buy_order_seed: VecSeed<OrderSeed<'h, BuyOrder>>,
    sell_order_seed: VecSeed<OrderSeed<'h, SellOrder>>,
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'a, 'b>>,
}

impl<'h, 'a, 'b, 'v> TradingTerminalSeed<'h, 'a, 'b, 'v> {
    fn new(
        order_holder: &'h OrderHolder,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'a BankRegistry,
        wallet_registry: &'b WalletRegistry,
    ) -> Self {
        Self {
            buy_order_seed: VecSeed::new(OrderSeed::new(order_holder)),
            sell_order_seed: VecSeed::new(OrderSeed::new(order_holder)),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(
                objective_vault,
                bank_registry,
                wallet_registry,
            )),
        }
    }
}

impl TradingTerminal {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            id: ModuleId::new_v4(),
            buy_offers: vec![],
            sell_offers: vec![],
            buy_orders: vec![],
            sell_orders: vec![],
            operator: None,
            operational_wallet: None,
        })
    }
}

impl DynSerialize for TradingTerminal {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

struct Console<'a> {
    id: ModuleId,
    buy_offers: &'a mut Vec<BuyOffer>,
    sell_offers: &'a mut Vec<SellOffer>,
    operational_wallet: &'a mut Option<WalletId>,
}

impl<'a> ModuleConsole for Console<'a> {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn interact(&mut self) -> bool {
        todo!()
    }

    fn in_progress(&self) -> bool {
        todo!()
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        None
    }

    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole> {
        todo!()
    }

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole> {
        todo!()
    }

    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole> {
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }

    fn trading_admin_console(&self) -> Option<&dyn AdminTradingConsole> {
        todo!()
    }

    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn AdminTradingConsole> {
        Some(self)
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        todo!()
    }

    fn storages_by_role_mut(&mut self, role: StorageRole) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn safes(&self) -> &[ItemSafe] {
        todo!()
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        todo!()
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }
}

impl Module for TradingTerminal {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        CORE_PACKAGE_ID.into()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        environment_context: &mut EnvironmentContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        let mut console = Console {
            id: self.id,
            buy_offers: &mut self.buy_offers,
            sell_offers: &mut self.sell_offers,
            operational_wallet: &mut self.operational_wallet,
        };

        if let Some(operator) = &mut self.operator {
            operator.proceed(
                &mut rng(),
                &mut console,
                this_vessel.console(),
                environment_context,
                decider_vault,
                logger,
            )
        }
    }

    fn active(&self) -> bool {
        todo!()
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        collector.exit_module();
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        &[]
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        &[]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        if self
            .operator
            .as_ref()
            .map(|p| p.id() == id)
            .unwrap_or(false)
        {
            self.operator.take()
        } else {
            None
        }
    }

    fn insert_person(&mut self, person: Person) -> bool {
        if self.operator.is_none() {
            self.operator = Some(person);
            true
        } else {
            false
        }
    }

    fn free_person_slots_count(&self) -> usize {
        const CAPACITY: usize = 1;
        CAPACITY - self.operator.iter().len()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        self.operator
            .as_ref()
            .map(|p| p.id() == id)
            .unwrap_or(false)
    }

    fn persons(&self) -> &[Person] {
        self.operator
            .as_ref()
            .map(std::slice::from_ref)
            .unwrap_or(&[])
    }

    fn persons_mut(&mut self) -> &mut [Person] {
        self.operator
            .as_mut()
            .map(std::slice::from_mut)
            .unwrap_or(&mut [])
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        todo!()
    }

    fn storages_by_role_mut(&mut self, role: StorageRole) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn safes(&self) -> &[ItemSafe] {
        &[]
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        &[]
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        &[]
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }

    fn docking_connectors(&self) -> &[DockingConnector] {
        &[]
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        Some(self)
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        Some(self)
    }
}

impl TradingConsole for TradingTerminal {
    fn buy_offers(&self) -> &[BuyOffer] {
        &self.buy_offers
    }

    fn sell_offers(&self) -> &[SellOffer] {
        &self.sell_offers
    }

    fn place_buy_order(
        &mut self,
        customer_wallet: &mut Wallet,
        vessel_to_buy_from: VesselId,
        offer: &BuyOffer,
        count: ItemCount,
    ) -> Result<WeakBuyOrder, PlaceBuyOrderError> {
        let offer: &BuyOffer = self
            .buy_offers
            .iter()
            .find(|BuyOffer { id, .. }| *id == offer.id)
            .unwrap();

        if !offer.count_range.contains(&count) {
            return Err(PlaceBuyOrderError::CountIsNotInRange);
        }

        let total_price = offer.price_per_unit.clone() * count;

        let mut pledge_wallet = Wallet::new();
        customer_wallet
            .transfer_to(&mut pledge_wallet, total_price)
            .unwrap();

        let (order, weak_order) = BuyOrder::new(
            pledge_wallet,
            customer_wallet.id().clone(),
            vessel_to_buy_from,
            offer.item.clone(),
            count,
        );

        self.buy_orders.push(order);
        Ok(weak_order)
    }

    fn place_sell_order(
        &mut self,
        wallet_registry: &WalletRegistry,
        vessel_to_sell_to: VesselId,
        offer: &SellOffer,
        count: ItemCount,
    ) -> Result<WeakSellOrder, PlaceSellOrderError> {
        let offer: &SellOffer = self
            .sell_offers
            .iter()
            .find(|SellOffer { id, .. }| *id == offer.id)
            .unwrap();

        if !offer.count_range.contains(&count) {
            return Err(PlaceSellOrderError::CountIsNotInRange);
        }

        let total_price = offer.price_per_unit.clone() * count;

        let mut pledge_wallet = Wallet::new();
        let operational_wallet = wallet_registry
            .get(&self.operational_wallet.unwrap())
            .unwrap();
        let operational_wallet = operational_wallet.upgrade().unwrap();
        let mut operational_wallet = operational_wallet.borrow_mut();

        operational_wallet
            .transfer_to(&mut pledge_wallet, total_price)
            .unwrap();

        let (order, weak_order) = SellOrder::new(
            pledge_wallet,
            self.operational_wallet.unwrap().clone(),
            vessel_to_sell_to,
            offer.item.clone(),
            count,
        );

        self.sell_orders.push(order);
        Ok(weak_order)
    }

    fn dry_place_buy_order(
        &self,
        customer_wallet: &Wallet,
        offer: &BuyOffer,
        count: ItemCount,
    ) -> Result<(), PlaceBuyOrderError> {
        match self
            .buy_offers
            .iter()
            .find(|BuyOffer { id, .. }| *id == offer.id)
        {
            None => Err(PlaceBuyOrderError::OfferNotFound),
            Some(offer) => {
                if !offer.count_range.contains(&count) {
                    return Err(PlaceBuyOrderError::CountIsNotInRange);
                }

                if !customer_wallet.contains(offer.price_per_unit.clone() * count) {
                    return Err(PlaceBuyOrderError::NotEnoughMoneyInCustomerWallet);
                }

                Ok(())
            }
        }
    }

    fn dry_place_sell_order(
        &self,
        wallet_registry: &WalletRegistry,
        offer: &SellOffer,
        count: ItemCount,
    ) -> Result<(), PlaceSellOrderError> {
        match self
            .sell_offers
            .iter()
            .find(|SellOffer { id, .. }| *id == offer.id)
        {
            None => Err(PlaceSellOrderError::OfferNotFound),
            Some(offer) => {
                if !offer.count_range.contains(&count) {
                    return Err(PlaceSellOrderError::CountIsNotInRange);
                }

                if self.operational_wallet.is_none() {
                    return Err(PlaceSellOrderError::EmptyOperationalWallet);
                }

                if !wallet_registry
                    .get(&self.operational_wallet.unwrap())
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow_mut()
                    .contains(offer.price_per_unit.clone() * count)
                {
                    return Err(PlaceSellOrderError::NotEnoughMoneyInOperationalWallet);
                }

                Ok(())
            }
        }
    }

    fn buy_vessel_offers(&self) -> &[BuyVesselOffer] {
        &[]
    }

    fn place_buy_vessel_order(
        &mut self,
        offer: &BuyVesselOffer,
        count: usize,
    ) -> Option<WeakBuyVesselOrder> {
        todo!()
    }

    fn buy_custom_vessel_offer(&self) -> Option<&BuyCustomVesselOffer> {
        None
    }

    fn estimate_buy_custom_vessel_order(
        &self,
        capabilities: BTreeSet<ModuleCapability>,
        primary_capabilities: BTreeSet<ModuleCapability>,
        count: usize,
    ) -> Option<BuyCustomVesselOrderEstimate> {
        todo!()
    }

    fn place_buy_custom_vessel_order(
        &mut self,
        customer_wallet: &mut Wallet,
        capabilities: BTreeSet<ModuleCapability>,
        primary_capabilities: BTreeSet<ModuleCapability>,
        count: usize,
    ) -> Result<WeakBuyCustomVesselOrder, NotEnoughMoneyInWallet> {
        todo!()
    }
}

impl<'a> AdminTradingConsole for Console<'a> {
    fn set_operational_wallet(&mut self, wallet: WalletId) {
        *self.operational_wallet = Some(wallet);
    }

    fn place_buy_offer(
        &mut self,
        item: ItemId,
        count_range: RangeInclusive<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&BuyOffer> {
        assert!(count_range.is_valid());
        Some(self.buy_offers.push_mut(BuyOffer {
            id: OfferId::new_v4(),
            item,
            count_range,
            price_per_unit,
        }))
    }

    fn update_buy_offer(
        &mut self,
        id: OfferId,
        item: ItemId,
        count_range: RangeInclusive<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&BuyOffer> {
        assert!(count_range.is_valid());
        if let Some(x) = self.buy_offers.iter_mut().find(|o| o.id == id) {
            x.item = item;
            x.count_range = count_range;
            x.price_per_unit = price_per_unit;
            Some(x)
        } else {
            None
        }
    }

    fn place_buy_vessel_offer(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        price_per_unit: Money,
    ) -> Option<&BuyOffer> {
        todo!()
    }

    fn place_sell_offer(
        &mut self,
        item: ItemId,
        count_range: RangeInclusive<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&SellOffer> {
        assert!(count_range.is_valid());
        let offer = SellOffer {
            id: OfferId::new_v4(),
            item,
            count_range,
            price_per_unit,
        };
        self.sell_offers.push(offer);
        self.sell_offers.last()
    }

    fn update_sell_offer(
        &mut self,
        id: OfferId,
        item: ItemId,
        count_range: RangeInclusive<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&SellOffer> {
        assert!(count_range.is_valid());
        if let Some(x) = self.sell_offers.iter_mut().find(|o| o.id == id) {
            x.item = item;
            x.count_range = count_range;
            x.price_per_unit = price_per_unit;
            Some(x)
        } else {
            None
        }
    }

    fn place_buy_custom_vessel_offer(
        &mut self,
        capabilities: BTreeMap<ModuleCapability, Money>,
        primary_capabilities: BTreeMap<ModuleCapability, Money>,
    ) -> BuyCustomVesselOffer {
        todo!()
    }

    fn buy_offers(&self) -> &[BuyOffer] {
        todo!()
    }

    fn sell_offers(&self) -> &[SellOffer] {
        todo!()
    }

    fn buy_orders(&self) -> &[BuyOrder] {
        todo!()
    }

    fn sell_orders(&self) -> &[SellOrder] {
        todo!()
    }

    fn buy_vessel_orders(&self) -> &[BuyVesselOrder] {
        &[]
    }

    fn buy_custom_vessel_orders(&self) -> &[BuyCustomVesselOrder] {
        &[]
    }
}

pub(crate) struct TradingTerminalDynSeed {
    order_holder: Rc<OrderHolder>,
    objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    wallet_registry: Rc<WalletRegistry>,
}

impl TradingTerminalDynSeed {
    pub(crate) fn new(
        order_holder: Rc<OrderHolder>,
        objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
        wallet_registry: Rc<WalletRegistry>,
    ) -> Self {
        Self {
            order_holder,
            objective_vault,
            bank_registry,
            wallet_registry,
        }
    }
}

impl DynDeserializeSeed<dyn Module> for TradingTerminalDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: TradingTerminal = from_intermediate_seed(
            TradingTerminalSeed::new(
                &self.order_holder,
                &self.objective_vault,
                &self.bank_registry,
                &self.wallet_registry,
            ),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TradingTerminalFactory {}

impl ModuleFactory for TradingTerminalFactory {
    fn create(&self, item_vault: Rc<ItemVault>, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(TradingTerminal {
            id: ModuleId::new_v4(),
            buy_offers: vec![],
            sell_offers: vec![],
            buy_orders: vec![],
            sell_orders: vec![],
            operator: None,
            operational_wallet: None,
        })
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for TradingTerminalFactory {
    fn type_id(&self) -> ModuleTypeId {
        TYPE_ID.into()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        &[]
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        &[]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }
}

impl DynSerialize for TradingTerminalFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct TradingTerminalFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for TradingTerminalFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<TradingTerminalFactory> =
            from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
