use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{
    BuyOffer, ItemCount, ItemStorage, SellOffer, WeakBuyOrder, WeakSellOrder,
};
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId, TradingConsole,
};
use dudes_in_space_api::person::{
    Logger, ObjectiveDeciderVault, Person, PersonId, StatusCollector,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory};
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId};
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;

static TYPE_ID: &str = "UnmannedTradingTerminal";
static FACTORY_TYPE_ID: &str = "UnmannedTradingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::TradingTerminal];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::TradingTerminal];

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UnmannedTradingTerminal {
    id: ModuleId,
    buy_offers: Vec<BuyOffer>,
    sell_offers: Vec<SellOffer>,
}

impl UnmannedTradingTerminal {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            id: ModuleId::new_v4(),
            buy_offers: vec![],
            sell_offers: vec![],
        })
    }
}

impl DynSerialize for UnmannedTradingTerminal {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl Module for UnmannedTradingTerminal {
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
        PRIMARY_CAPABILITIES
    }

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        environment_context: &mut EnvironmentContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        collector.exit_module();
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
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
        &[]
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        Some(self)
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }
}

impl TradingConsole for UnmannedTradingTerminal {
    fn buy_offers(&self) -> &[BuyOffer] {
        &self.buy_offers
    }

    fn sell_offers(&self) -> &[SellOffer] {
        &self.sell_offers
    }

    fn place_buy_order(&mut self, offer: &BuyOffer, count: ItemCount) -> Option<WeakBuyOrder> {
        todo!()
    }

    fn place_sell_order(&mut self, offer: &SellOffer, count: ItemCount) -> Option<WeakSellOrder> {
        todo!()
    }
}

pub(crate) struct UnmannedTradingTerminalDynSeed;

impl DynDeserializeSeed<dyn Module> for UnmannedTradingTerminalDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: UnmannedTradingTerminal =
            from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UnmannedTradingTerminalFactory {}

impl ModuleFactory for UnmannedTradingTerminalFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, recipe: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(UnmannedTradingTerminal {
            id: ModuleId::new_v4(),
            buy_offers: vec![],
            sell_offers: vec![],
        })
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn output_primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }
}

impl DynSerialize for UnmannedTradingTerminalFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct UnmannedTradingTerminalFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for UnmannedTradingTerminalFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<UnmannedTradingTerminalFactory> =
            from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}
