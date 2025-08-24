use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::BankRegistry;
use dudes_in_space_api::item::{ItemCount, ItemSafe, ItemStorage};
use dudes_in_space_api::module::{
    CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, ModuleTypeId, PackageId, TradingAdminConsole, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::trade::{
    BuyCustomVesselOffer, BuyOffer, BuyOrder, BuyVesselOffer, OrderHolder, OrderSeed, SellOffer,
    SellOrder, WeakBuyCustomVesselOrderEstimate, WeakBuyOrder, WeakBuyVesselOrder, WeakSellOrder,
};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, VecSeed,
    from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;
use crate::CORE_PACKAGE_ID;

static TYPE_ID: &str = "TradingTerminal";
static FACTORY_TYPE_ID: &str = "TradingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::TradingTerminal];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::TradingTerminal];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::trading_terminal::TradingTerminalSeed::<'h,'b,'v>)]
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
}

struct TradingTerminalSeed<'h, 'b, 'v> {
    buy_order_seed: VecSeed<OrderSeed<'h, BuyOrder>>,
    sell_order_seed: VecSeed<OrderSeed<'h, SellOrder>>,
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'b>>,
}

impl<'h, 'b, 'v> TradingTerminalSeed<'h, 'b, 'v> {
    fn new(
        order_holder: &'h OrderHolder,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'b BankRegistry,
    ) -> Self {
        Self {
            buy_order_seed: VecSeed::new(OrderSeed::new(order_holder)),
            sell_order_seed: VecSeed::new(OrderSeed::new(order_holder)),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(objective_vault, bank_registry)),
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
    buy_offers: &'a [BuyOffer],
    sell_offers: &'a [SellOffer],
}

impl<'a> ModuleConsole for Console<'a> {
    fn id(&self) -> ModuleId {
        todo!()
    }

    fn type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn interact(&mut self) -> bool {
        todo!()
    }

    fn in_progress(&self) -> bool {
        todo!()
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        todo!()
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
        todo!()
    }

    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn TradingAdminConsole> {
        todo!()
    }

    fn storages(&self) -> &[ItemStorage] {
        todo!()
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
            buy_offers: &self.buy_offers,
            sell_offers: &self.sell_offers,
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
        collector.exit_module();
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        todo!()
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        todo!()
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        todo!()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        todo!()
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        todo!()
    }

    fn insert_person(&mut self, person: Person) -> bool {
        todo!()
    }

    fn free_person_slots_count(&self) -> usize {
        todo!()
    }

    fn contains_person(&self, id: PersonId) -> bool {
        todo!()
    }

    fn persons(&self) -> &[Person] {
        &[]
    }

    fn storages(&self) -> &[ItemStorage] {
        todo!()
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
        todo!()
    }
}

impl TradingConsole for TradingTerminal {
    fn buy_offers(&self) -> &[BuyOffer] {
        &self.buy_offers
    }

    fn sell_offers(&self) -> &[SellOffer] {
        &self.sell_offers
    }

    fn buy_vessel_offers(&self) -> &[BuyVesselOffer] {
        todo!()
    }

    fn place_buy_order(&mut self, offer: &BuyOffer, count: ItemCount) -> Option<WeakBuyOrder> {
        todo!()
    }

    fn place_sell_order(&mut self, offer: &SellOffer, count: ItemCount) -> Option<WeakSellOrder> {
        todo!()
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
    ) -> Option<WeakBuyCustomVesselOrderEstimate> {
        todo!()
    }

    fn place_buy_custom_vessel_order(
        &mut self,
        capabilities: BTreeSet<ModuleCapability>,
        primary_capabilities: BTreeSet<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyVesselOrder> {
        todo!()
    }
}

pub(crate) struct TradingTerminalDynSeed {
    order_holder: Rc<OrderHolder>,
    objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
}

impl TradingTerminalDynSeed {
    pub(crate) fn new(
        order_holder: Rc<OrderHolder>,
        objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
    ) -> Self {
        Self {
            order_holder,
            objective_vault,
            bank_registry,
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
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(TradingTerminal {
            id: ModuleId::new_v4(),
            buy_offers: vec![],
            sell_offers: vec![],
            buy_orders: vec![],
            sell_orders: vec![],
            operator: None,
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
