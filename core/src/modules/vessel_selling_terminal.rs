use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::item::{
    BuyOffer, BuyVesselOffer, ItemCount, ItemSafe, ItemStorage, SellOffer, WeakBuyOrder,
    WeakBuyVesselManualOrderEstimate, WeakBuyVesselOrder, WeakSellOrder,
};
use dudes_in_space_api::module::{
    Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId, TradingConsole,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use serde_intermediate::{Intermediate, to_intermediate};
use std::error::Error;
use std::rc::Rc;

static TYPE_ID: &str = "VesselSellingTerminal";
static FACTORY_TYPE_ID: &str = "VesselSellingTerminalFactory";
static CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[ModuleCapability::VesselSellingTerminal];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::vessel_selling_terminal::VesselSellingTerminalSeed::<'v>)]
struct VesselSellingTerminal {
    id: ModuleId,
    offers: Vec<BuyVesselOffer>,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    operator: Option<Person>,
}

struct VesselSellingTerminalSeed<'v> {
    person_seed: TaggedOptionSeed<PersonSeed<'v>>,
}

impl<'v> VesselSellingTerminalSeed<'v> {
    fn new(objective_vault: &'v DynDeserializeSeedVault<dyn DynObjective>) -> Self {
        Self {
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

impl TradingConsole for VesselSellingTerminal {
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
    objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
}

impl VesselSellingTerminalDynSeed {
    pub(crate) fn new(objective_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>) -> Self {
        Self { objective_vault }
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
            VesselSellingTerminalSeed::new(&self.objective_vault),
            &intermediate,
        )
        .map_err(|e| e.to_string())?;
        Ok(Box::new(r))
    }
}
