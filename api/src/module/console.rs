use crate::item::{
    BuyOffer, BuyVesselOffer, ItemCount, ItemId, ItemSafe, ItemStorage, SellOffer, WeakBuyOrder,
    WeakBuyVesselManualOrderEstimate, WeakBuyVesselOrder, WeakSellOrder,
};
use crate::module::module::ModuleId;
use crate::module::{ModuleCapability, ModuleStorage, ModuleTypeId, PackageId, ProcessToken};
use crate::person::{Money, Role};
use crate::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactoryOutputDescription, OutputItemRecipe,
};
use crate::utils::math::Vector;
use crate::utils::range::Range;
use crate::vessel::DockingClamp;
use std::collections::BTreeSet;

/// interface through which a person can interact with a module
pub trait ModuleConsole {
    /// common
    fn id(&self) -> ModuleId;
    fn type_id(&self) -> ModuleTypeId;
    fn package_id(&self) -> PackageId;
    fn capabilities(&self) -> &[ModuleCapability];
    fn primary_capabilities(&self) -> &[ModuleCapability];

    /// interact
    fn interact(&mut self) -> bool;
    fn in_progress(&self) -> bool;

    /// consoles
    fn crafting_console(&self) -> Option<&dyn CraftingConsole>;
    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole>;

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole>;
    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole>;

    fn trading_console(&self) -> Option<&dyn TradingConsole>;
    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole>;

    fn trading_admin_console(&self) -> Option<&dyn TradingAdminConsole>;
    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn TradingAdminConsole>;

    fn storages(&self) -> &[ItemStorage];
    fn storages_mut(&mut self) -> &mut [ItemStorage];

    fn safes(&self) -> &[ItemSafe];
    fn safes_mut(&mut self) -> &mut [ItemSafe];

    fn module_storages(&self) -> &[ModuleStorage];
    fn module_storages_mut(&mut self) -> &mut [ModuleStorage];

    fn docking_clamps(&self) -> &[DockingClamp];
    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp];
}

pub struct DefaultModuleConsole<'c, 'pc> {
    id: ModuleId,
    capabilities: &'c [ModuleCapability],
    primary_capabilities: &'pc [ModuleCapability],
}

impl<'c, 'pc, 'd> DefaultModuleConsole<'c, 'pc> {
    pub fn new(
        id: ModuleId,
        capabilities: &'c [ModuleCapability],
        primary_capabilities: &'pc [ModuleCapability],
    ) -> Self {
        Self {
            id,
            capabilities,
            primary_capabilities,
        }
    }
}

impl<'c, 'pc> ModuleConsole for DefaultModuleConsole<'c, 'pc> {
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
        self.capabilities
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        self.primary_capabilities
    }

    fn interact(&mut self) -> bool {
        false
    }

    fn in_progress(&self) -> bool {
        todo!()
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        None
    }

    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole> {
        None
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
}

pub trait ModuleInfoConsole {}

pub trait CraftingConsole {
    // returns index in array. TODO replace with uuid
    fn recipe_by_output_capability(&self, capability: ModuleCapability) -> Option<usize>;
    fn recipe_by_output_primary_capability(&self, capability: ModuleCapability) -> Option<usize>;
    fn recipe_by_output_item(&self, item: ItemId) -> Option<usize>;

    fn recipe_output_description(&self, index: usize) -> &dyn ModuleFactoryOutputDescription;
    fn recipe_item_output(&self, index: usize) -> Option<OutputItemRecipe>;

    // returns index in array. TODO replace with uuid
    fn has_resources_for_recipe(&self, index: usize) -> bool;
    fn active_recipe(&self) -> Option<usize>;
    /// inputs index in array. TODO replace with uuid
    /// deploy - if true will attach the produced module to this vessel, false - will store in a nearest module storage
    fn start(&mut self, index: usize, deploy: bool) -> Option<ProcessToken>;
    fn item_recipes(&self) -> &[ItemRecipe];
    fn input_item_recipes(&self) -> &[InputItemRecipe];
    fn output_item_recipes(&self) -> &[OutputItemRecipe];
    fn assembly_recipes(&self) -> &[AssemblyRecipe];
}

pub trait DockyardConsole {
    fn start(&mut self, modules: BTreeSet<ModuleId>) -> Option<ProcessToken>;
}

pub trait TradingConsole {
    fn buy_offers(&self) -> &[BuyOffer];
    fn buy_vessel_offers(&self) -> &[BuyVesselOffer];
    fn sell_offers(&self) -> &[SellOffer];
    fn place_buy_order(&mut self, offer: &BuyOffer, count: ItemCount) -> Option<WeakBuyOrder>;
    fn place_buy_vessel_order(
        &mut self,
        offer: &BuyVesselOffer,
        count: usize,
    ) -> Option<WeakBuyVesselOrder>;
    fn place_sell_order(&mut self, offer: &SellOffer, count: ItemCount) -> Option<WeakSellOrder>;
    
    fn caps_available_for_manual_order(&self) -> Vec<ModuleCapability>;

    fn primary_caps_available_for_manual_order(&self) -> Vec<ModuleCapability>;

    fn estimate_buy_vessel_manual_order(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyVesselManualOrderEstimate>;

    fn place_buy_vessel_manual_order(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        count: usize,
    ) -> Option<WeakBuyVesselOrder>;
}

pub trait TradingAdminConsole {
    fn place_buy_offer(
        &mut self,
        item: ItemId,
        count_range: Range<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&BuyOffer>;
    fn place_buy_vessel_offer(
        &mut self,
        primary_caps: Vec<ModuleCapability>,
        price_per_unit: Money,
    ) -> Option<&BuyOffer>;
    fn place_sell_offer(
        &mut self,
        item: ItemId,
        count_range: Range<ItemCount>,
        price_per_unit: Money,
    ) -> Option<&SellOffer>;
}

pub(crate) trait CaptainControlPanel {
    fn give_command(&self, _role: Role) {}
}

pub(crate) trait NavigatorControlPanel {
    fn scan(&self) {}

    fn plan_route(&self) {}
}

pub(crate) trait GunnerControlPanel {
    fn scan(&self) -> Vector<u32> {
        todo!()
    }

    fn fire_at(&self, _vessel_id: u32) {
        todo!()
    }
}
