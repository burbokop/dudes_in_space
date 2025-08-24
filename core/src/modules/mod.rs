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

pub(crate) use assembler::*;
pub(crate) use cargo_container::*;
pub(crate) use core_module::*;
pub(crate) use dockyard::*;
pub(crate) use fabricator::*;
pub(crate) use ore_manifold::*;
pub(crate) use personnel_area::*;
pub(crate) use plant_facility::*;
pub(crate) use shuttle::*;
pub(crate) use trading_terminal::*;
pub(crate) use vessel_selling_terminal::*;

use dudes_in_space_api::finance::BankRegistry;
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
    item_vault: Rc<ItemVault>,
    order_holder: Rc<OrderHolder>,
    process_token_context: Rc<ProcessTokenContext>,
) -> DynDeserializeSeedVault<dyn Module> {
    vault
        .with(PersonnelAreaDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
        ))
        .with(ShuttleDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
        ))
        .with(DockyardDynSeed::new(
            objective_seed_vault.clone(),
            bank_registry.clone(),
            process_token_context.clone(),
        ))
        .with(AssemblerDynSeed::new(
            factory_seed_vault,
            objective_seed_vault.clone(),
            bank_registry.clone(),
            item_vault.clone(),
            process_token_context.clone(),
        ))
        .with(CargoContainerDynSeed::new(item_vault.clone()))
        .with(TradingTerminalDynSeed::new(
            order_holder.clone(),
            objective_seed_vault.clone(),
            bank_registry.clone(),
        ))
        .with(VesselSellingTerminalDynSeed::new(
            order_holder,
            objective_seed_vault.clone(),
            bank_registry.clone(),
        ))
        .with(FabricatorDynSeed::new(
            objective_seed_vault,
            bank_registry.clone(),
            item_vault,
            process_token_context,
        ))
        .with(PlantFacilityDynSeed {})
        .with(OreManifoldDynSeed {})
}
