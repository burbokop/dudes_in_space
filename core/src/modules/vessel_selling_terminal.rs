use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{ItemCount, ItemId, ItemSafe, ItemStorage};
use dudes_in_space_api::module::{
    CraftingConsole, DockyardConsole, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, ModuleTypeId, PackageId, TradingAdminConsole, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, Money, ObjectiveDeciderVault, Person, PersonId, PersonSeed,
    StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::trade::{
    BuyCustomVesselOffer, BuyOffer, BuyVesselOffer, BuyVesselOrder, OrderHolder, OrderSeed,
    SellOffer, WeakBuyCustomVesselOrderEstimate, WeakBuyOrder, WeakBuyVesselOrder, WeakSellOrder,
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
use std::collections::BTreeSet;
use std::error::Error;
use std::rc::Rc;

static TYPE_ID: &str = "VesselSellingTerminal";
static FACTORY_TYPE_ID: &str = "VesselSellingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::vessel_selling_terminal::VesselSellingTerminalSeed::<'h, 'v>)]
struct VesselSellingTerminal {
    id: ModuleId,
    offers: Vec<BuyVesselOffer>,
    capabilities_available_for_manual_order: BTreeSet<ModuleCapability>,
    primary_capabilities_available_for_manual_order: BTreeSet<ModuleCapability>,
    #[deserialize_seed_xxx(seed = self.seed.order_seed)]
    orders: Vec<BuyVesselOrder>,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}

struct VesselSellingTerminalSeed<'h, 'v> {
    order_seed: VecSeed<OrderSeed<'h, BuyVesselOrder>>,
    person_seed: TaggedOptionSeed<PersonSeed<'v>>,
}

impl<'h, 'v> VesselSellingTerminalSeed<'h, 'v> {
    fn new(
        order_holder: &'h OrderHolder,
        objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
    ) -> Self {
        Self {
            order_seed: VecSeed::new(OrderSeed::new(order_holder)),
            person_seed: TaggedOptionSeed::new(PersonSeed::new(objective_vault)),
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
    capabilities_available_for_manual_order: &'a mut BTreeSet<ModuleCapability>,
    primary_capabilities_available_for_manual_order: &'a mut BTreeSet<ModuleCapability>,
    orders: &'a [BuyVesselOrder],
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

    fn set_capabilities_available_for_manual_order(&mut self, caps: BTreeSet<ModuleCapability>) {
        *self.capabilities_available_for_manual_order = caps
    }

    fn set_primary_capabilities_available_for_manual_order(
        &mut self,
        caps: BTreeSet<ModuleCapability>,
    ) {
        *self.primary_capabilities_available_for_manual_order = caps;
    }

    fn orders(&self) -> &[BuyVesselOrder] {
        self.orders
    }
}

impl Module for VesselSellingTerminal {
    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
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
            capabilities_available_for_manual_order: &mut self
                .capabilities_available_for_manual_order,
            primary_capabilities_available_for_manual_order: &mut self
                .primary_capabilities_available_for_manual_order,
            orders: &self.orders,
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
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
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

    fn buy_custom_vessel_offer(&self) -> Option<BuyCustomVesselOffer> {
        todo!()
    }

    fn estimate_buy_custom_vessel_order(
        &mut self,
        primary_capabilities: Vec<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyCustomVesselOrderEstimate> {
        todo!()
    }

    fn place_buy_custom_vessel_order(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyVesselOrder> {
        todo!()
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
            capabilities_available_for_manual_order: BTreeSet::new(),
            primary_capabilities_available_for_manual_order: BTreeSet::new(),
            orders: vec![],
            operator: None,
        })
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for VesselSellingTerminalFactory {
    fn type_id(&self) -> ModuleTypeId {
        todo!()
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
    objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
}

impl VesselSellingTerminalDynSeed {
    pub(crate) fn new(
        order_holder: Rc<OrderHolder>,
        objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    ) -> Self {
        Self {
            order_holder,
            objective_vault,
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
            VesselSellingTerminalSeed::new(&self.order_holder, &self.objective_vault),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(r))
    }
}
