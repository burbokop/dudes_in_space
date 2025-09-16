use crate::CORE_PACKAGE_ID;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{
    BankRegistry, Money, NotEnoughMoneyInWallet, Wallet, WalletRegistry,
};
use dudes_in_space_api::item::{ItemCount, ItemId, ItemSafe, ItemStorage};
use dudes_in_space_api::module::{
    CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, ModuleTypeId, PackageId, ProcessTokenContext, TradingAdminConsole,
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
    BuyCustomVesselOffer, BuyCustomVesselOrder, BuyCustomVesselOrderEstimate,
    BuyCustomVesselOrderSeed, BuyOffer, BuyOrder, BuyVesselOffer, BuyVesselOrder, OfferId,
    OrderHolder, OrderSeed, SellOffer, SellOrder, WeakBuyCustomVesselOrder, WeakBuyOrder,
    WeakBuyVesselOrder, WeakSellOrder,
};
use dudes_in_space_api::utils::range::Range;
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, VecSeed,
    from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::Serialize;
use serde_intermediate::{Intermediate, to_intermediate};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::rc::Rc;

static TYPE_ID: &str = "VesselSellingTerminal";
static FACTORY_TYPE_ID: &str = "VesselSellingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::VesselSellingTerminal,
    ModuleCapability::PersonnelRoom,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::vessel_selling_terminal::VesselSellingTerminalSeed::<'h,'c,'a,'b, 'v>)]
struct VesselSellingTerminal {
    id: ModuleId,
    offers: Vec<BuyVesselOffer>,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    buy_custom_vessel_offer: Option<BuyCustomVesselOffer>,
    #[deserialize_seed_xxx(seed = self.seed.order_seed)]
    orders: Vec<BuyVesselOrder>,
    #[deserialize_seed_xxx(seed = self.seed.custom_order_seed)]
    #[serde(default)]
    custom_orders: Vec<BuyCustomVesselOrder>,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}

struct VesselSellingTerminalSeed<'h, 'c, 'a, 'b, 'v> {
    order_seed: VecSeed<OrderSeed<'h, BuyVesselOrder>>,
    custom_order_seed: VecSeed<BuyCustomVesselOrderSeed<'h, 'c>>,
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'a, 'b>>,
}

impl<'h, 'c, 'a, 'b, 'v> VesselSellingTerminalSeed<'h, 'c, 'a, 'b, 'v> {
    fn new(
        order_holder: &'h OrderHolder,
        process_token_context: &'c ProcessTokenContext,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'a BankRegistry,
        wallet_registry: &'b WalletRegistry,
    ) -> Self {
        Self {
            order_seed: VecSeed::new(OrderSeed::new(order_holder)),
            custom_order_seed: VecSeed::new(BuyCustomVesselOrderSeed::new(
                order_holder,
                process_token_context,
            )),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(
                objective_vault,
                bank_registry,
                wallet_registry,
            )),
        }
    }
}

impl DynSerialize for VesselSellingTerminal {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

struct Console<'a> {
    id: ModuleId,
    offers: &'a [BuyVesselOffer],
    buy_custom_vessel_offer: &'a mut Option<BuyCustomVesselOffer>,
    orders: &'a [BuyVesselOrder],
    custom_orders: &'a [BuyCustomVesselOrder],
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

    fn trading_admin_console(&self) -> Option<&dyn TradingAdminConsole> {
        Some(self)
    }

    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn TradingAdminConsole> {
        Some(self)
    }

    fn storages(&self) -> &[ItemStorage] {
        &[]
    }

    fn storages_mut(&mut self) -> &mut [ItemStorage] {
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

impl<'a> TradingAdminConsole for Console<'a> {
    fn place_buy_offer(
        &mut self,
        item: ItemId,
        count_range: Range<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&BuyOffer> {
        todo!()
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
        count_range: Range<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&SellOffer> {
        todo!()
    }

    fn place_buy_custom_vessel_offer(
        &mut self,
        capabilities: BTreeMap<ModuleCapability, Money>,
        primary_capabilities: BTreeMap<ModuleCapability, Money>,
    ) -> BuyCustomVesselOffer {
        let offer = BuyCustomVesselOffer {
            id: OfferId::new_v4(),
            available_capabilities: capabilities,
            available_primary_capabilities: primary_capabilities,
        };
        *self.buy_custom_vessel_offer = Some(offer.clone());
        offer
    }

    fn buy_orders(&self) -> &[BuyOrder] {
        todo!()
    }

    fn sell_orders(&self) -> &[SellOrder] {
        todo!()
    }

    fn buy_vessel_orders(&self) -> &[BuyVesselOrder] {
        self.orders
    }

    fn buy_custom_vessel_orders(&self) -> &[BuyCustomVesselOrder] {
        self.custom_orders
    }
}

impl Module for VesselSellingTerminal {
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
            offers: &self.offers,
            buy_custom_vessel_offer: &mut self.buy_custom_vessel_offer,
            orders: &self.orders,
            custom_orders: &self.custom_orders,
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

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        if let Some(operator) = &self.operator {
            operator.collect_status(collector);
        }
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
        match self.operator.as_ref() {
            None => &[],
            Some(person) => std::slice::from_ref(person),
        }
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

impl TradingConsole for VesselSellingTerminal {
    fn buy_offers(&self) -> &[BuyOffer] {
        &[]
    }

    fn sell_offers(&self) -> &[SellOffer] {
        &[]
    }

    fn place_buy_order(&mut self, offer: &BuyOffer, count: ItemCount) -> Option<WeakBuyOrder> {
        todo!()
    }

    fn place_sell_order(&mut self, offer: &SellOffer, count: ItemCount) -> Option<WeakSellOrder> {
        todo!()
    }

    fn buy_vessel_offers(&self) -> &[BuyVesselOffer] {
        &self.offers
    }

    fn place_buy_vessel_order(
        &mut self,
        offer: &BuyVesselOffer,
        count: usize,
    ) -> Option<WeakBuyVesselOrder> {
        todo!()
    }

    fn buy_custom_vessel_offer(&self) -> Option<&BuyCustomVesselOffer> {
        self.buy_custom_vessel_offer.as_ref()
    }

    fn estimate_buy_custom_vessel_order(
        &self,
        capabilities: BTreeSet<ModuleCapability>,
        primary_capabilities: BTreeSet<ModuleCapability>,
        count: usize,
    ) -> Option<BuyCustomVesselOrderEstimate> {
        let offer = self.buy_custom_vessel_offer.as_ref()?;

        let estimate = Money::sum_same_currency(std::iter::chain(
            capabilities
                .into_iter()
                .map(|cap| offer.available_capabilities.get(&cap).unwrap().clone()),
            primary_capabilities.into_iter().map(|cap| {
                offer
                    .available_primary_capabilities
                    .get(&cap)
                    .unwrap()
                    .clone()
            }),
        ))
        .unwrap();

        Some(BuyCustomVesselOrderEstimate {
            estimate: estimate.clone(),
            pledge: estimate / 2.,
        })
    }

    fn place_buy_custom_vessel_order(
        &mut self,
        customer_wallet: &mut Wallet,
        capabilities: BTreeSet<ModuleCapability>,
        primary_capabilities: BTreeSet<ModuleCapability>,
        count: usize,
    ) -> Result<WeakBuyCustomVesselOrder, NotEnoughMoneyInWallet> {
        let estimate = self
            .estimate_buy_custom_vessel_order(
                capabilities.clone(),
                primary_capabilities.clone(),
                count,
            )
            .unwrap();

        let x = estimate.pledge;

        let mut pledge_wallet = Wallet::new();

        customer_wallet.transfer_to(&mut pledge_wallet, x)?;

        let (weak_order, order) = BuyCustomVesselOrder::new(
            pledge_wallet,
            customer_wallet.id().clone(),
            capabilities,
            primary_capabilities,
            count,
        );

        self.custom_orders.push(order);
        Ok(weak_order)
    }
}

#[derive(Debug)]
pub(crate) struct VesselSellingTerminalFactory {}

impl DynSerialize for VesselSellingTerminalFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl ModuleFactory for VesselSellingTerminalFactory {
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(VesselSellingTerminal {
            id: ModuleId::new_v4(),
            offers: vec![],
            buy_custom_vessel_offer: None,
            orders: vec![],
            custom_orders: vec![],
            operator: None,
        })
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for VesselSellingTerminalFactory {
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

pub(crate) struct VesselSellingTerminalFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for VesselSellingTerminalFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        todo!()
    }
}

pub(crate) struct VesselSellingTerminalDynSeed {
    order_holder: Rc<OrderHolder>,
    process_token_context: Rc<ProcessTokenContext>,
    objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    wallet_registry: Rc<WalletRegistry>,
}

impl VesselSellingTerminalDynSeed {
    pub(crate) fn new(
        order_holder: Rc<OrderHolder>,
        process_token_context: Rc<ProcessTokenContext>,
        objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
        wallet_registry: Rc<WalletRegistry>,
    ) -> Self {
        Self {
            order_holder,
            process_token_context,
            objective_vault,
            bank_registry,
            wallet_registry,
        }
    }
}

impl DynDeserializeSeed<dyn Module> for VesselSellingTerminalDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let r: VesselSellingTerminal = from_intermediate_seed(
            VesselSellingTerminalSeed::new(
                &self.order_holder,
                &self.process_token_context,
                &self.objective_vault,
                &self.bank_registry,
                &self.wallet_registry,
            ),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(r))
    }
}
