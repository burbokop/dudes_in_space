use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{
    BuyOffer, BuyVesselOffer, ItemCount, ItemSafe, ItemStorage, SellOffer, WeakBuyOrder,
    WeakBuyVesselManualOrderEstimate, WeakBuyVesselOrder, WeakSellOrder,
};
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId, TradingConsole,
};
use dudes_in_space_api::person::{
    Logger, ObjectiveDeciderVault, Person, PersonId, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde_intermediate::Intermediate;
use std::error::Error;

static TYPE_ID: &str = "UnmannedVesselSellingTerminal";
static FACTORY_TYPE_ID: &str = "UnmannedVesselSellingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];

#[derive(Debug)]
struct UnmannedVesselSellingTerminal {
    offers: Vec<BuyVesselOffer>,
}

impl DynSerialize for UnmannedVesselSellingTerminal {
    fn type_id(&self) -> TypeId {
        todo!()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl Module for UnmannedVesselSellingTerminal {
    fn id(&self) -> ModuleId {
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

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        environment_context: &mut EnvironmentContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        todo!()
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        todo!()
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

impl TradingConsole for UnmannedVesselSellingTerminal {
    fn buy_offers(&self) -> &[BuyOffer] {
        &[]
    }

    fn sell_offers(&self) -> &[SellOffer] {
        &[]
    }

    fn buy_vessel_offers(&self) -> &[BuyVesselOffer] {
        &self.offers
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

    fn estimate_buy_vessel_manual_order(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyVesselManualOrderEstimate> {
        todo!()
    }

    fn place_buy_vessel_manual_order(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyVesselOrder> {
        todo!()
    }

    fn caps_available_for_manual_order(&self) -> Vec<ModuleCapability> {
        todo!()
    }

    fn primary_caps_available_for_manual_order(&self) -> Vec<ModuleCapability> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct UnmannedVesselSellingTerminalFactory {}

impl DynSerialize for UnmannedVesselSellingTerminalFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.into()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        todo!()
    }
}

impl ModuleFactory for UnmannedVesselSellingTerminalFactory {
    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        todo!()
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for UnmannedVesselSellingTerminalFactory {
    fn type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        todo!()
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        todo!()
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

pub(crate) struct UnmannedVesselSellingTerminalFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for UnmannedVesselSellingTerminalFactoryDynSeed {
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

pub(crate) struct UnmannedVesselSellingTerminalDynSeed;

impl DynDeserializeSeed<dyn Module> for UnmannedVesselSellingTerminalDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.into()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        todo!()
    }
}
