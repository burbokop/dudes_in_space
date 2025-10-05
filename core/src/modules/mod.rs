mod assembler;
mod cargo_container;
mod core_module;
mod dockyard;
mod fabricator;
mod ore_manifold;
mod personnel_area;
mod plant_facility;
mod shuttle;
mod trading_terminal;
mod vessel_selling_terminal;

#[allow(unused_imports)]
pub use assembler::*;
#[allow(unused_imports)]
pub use cargo_container::*;
#[allow(unused_imports)]
pub use core_module::*;
#[allow(unused_imports)]
pub use dockyard::*;
#[allow(unused_imports)]
pub use fabricator::*;
#[allow(unused_imports)]
pub use ore_manifold::*;
#[allow(unused_imports)]
pub use personnel_area::*;
#[allow(unused_imports)]
pub use plant_facility::*;
#[allow(unused_imports)]
pub use shuttle::*;
#[allow(unused_imports)]
pub use trading_terminal::*;
#[allow(unused_imports)]
pub use vessel_selling_terminal::*;

use dudes_in_space_api::finance::{BankRegistry, WalletRegistry};
use dudes_in_space_api::item::ItemVault;
use dudes_in_space_api::module::{Module, ProcessTokenContext};
use dudes_in_space_api::person::DynObjective;
use dudes_in_space_api::recipe::ModuleFactory;
use dudes_in_space_api::trade::OrderHolder;
use dyn_serde::DynDeserializeSeedVault;
use std::rc::Rc;

pub fn register_module_factories(
    vault: DynDeserializeSeedVault<dyn ModuleFactory>,
) -> DynDeserializeSeedVault<dyn ModuleFactory> {
    vault
        .with(ShuttleFactoryDynSeed)
        .with(DockyardFactoryDynSeed)
        .with(CargoContainerFactoryDynSeed)
        .with(TradingTerminalFactoryDynSeed)
        .with(VesselSellingTerminalFactoryDynSeed)
        .with(FabricatorFactoryDynSeed)
        .with(PlantFacilityFactoryDynSeed)
        .with(OreManifoldFactoryDynSeed)
}

pub fn register_modules(
    vault: DynDeserializeSeedVault<dyn Module>,
    factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    wallet_registry: Rc<WalletRegistry>,
    item_vault: Rc<ItemVault>,
    order_holder: Rc<OrderHolder>,
    process_token_context: Rc<ProcessTokenContext>,
) -> DynDeserializeSeedVault<dyn Module> {
    vault
        .with(PersonnelAreaDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
        ))
        .with(ShuttleDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
        ))
        .with(DockyardDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
            process_token_context.clone(),
        ))
        .with(AssemblerDynSeed::new(
            factory_seed_vault.clone(),
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
            item_vault.clone(),
            process_token_context.clone(),
        ))
        .with(CargoContainerDynSeed::new(item_vault.clone()))
        .with(TradingTerminalDynSeed::new(
            order_holder.clone(),
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
        ))
        .with(VesselSellingTerminalDynSeed::new(
            order_holder,
            process_token_context.clone(),
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
        ))
        .with(FabricatorDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
            item_vault.clone(),
            process_token_context.clone(),
        ))
        .with(PlantFacilityDynSeed::new(
            factory_seed_vault.clone(),
            objective_seed_vault.clone(),
            bank_registry.clone(),
            wallet_registry.clone(),
            item_vault.clone(),
            process_token_context.clone(),
        ))
        .with(OreManifoldDynSeed::new(
            factory_seed_vault,
            objective_seed_vault,
            bank_registry,
            wallet_registry,
            item_vault,
            process_token_context,
        ))
}

pub mod types {
    pub static ASSEMBLER: &str = crate::modules::assembler::TYPE_ID;
    pub static CARGO_CONTAINER: &str = crate::modules::cargo_container::TYPE_ID;
    pub static DOCKYARD: &str = crate::modules::dockyard::TYPE_ID;
    pub static FABRICATOR: &str = crate::modules::fabricator::TYPE_ID;
    pub static ORE_MANIFOLD: &str = crate::modules::ore_manifold::TYPE_ID;
    pub static PERSONNEL_AREA: &str = crate::modules::personnel_area::TYPE_ID;
    pub static PLANT_FACILITY: &str = crate::modules::plant_facility::TYPE_ID;
    pub static SHUTTLE: &str = crate::modules::shuttle::TYPE_ID;
    pub static TRADING_TERMINAL: &str = crate::modules::trading_terminal::TYPE_ID;
    pub static VESSEL_SELLING_TERMINAL: &str = crate::modules::vessel_selling_terminal::TYPE_ID;
}
