mod assembler;
mod cargo_container;
mod core_module;
mod dockyard;
mod personnel_area;
mod shuttle;

pub(crate) use assembler::*;
pub(crate) use cargo_container::*;
pub(crate) use core_module::*;
pub(crate) use dockyard::*;
pub(crate) use personnel_area::*;
pub(crate) use shuttle::*;

use dudes_in_space_api::module::{Module, ProcessTokenContext};
use dudes_in_space_api::person::DynObjective;
use dudes_in_space_api::recipe::ModuleFactory;
use dyn_serde::DynDeserializeSeedVault;
use std::rc::Rc;

pub fn register_module_factories(
    vault: DynDeserializeSeedVault<dyn ModuleFactory>,
) -> DynDeserializeSeedVault<dyn ModuleFactory> {
    vault
        .with(ShuttleFactoryDynSeed)
        .with(DockyardFactoryDynSeed)
        .with(CargoContainerFactoryDynSeed)
}

pub fn register_modules(
    vault: DynDeserializeSeedVault<dyn Module>,
    factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
    objective_seed_vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    process_token_context: Rc<ProcessTokenContext>,
) -> DynDeserializeSeedVault<dyn Module> {
    vault
        .with(PersonnelAreaDynSeed::new(objective_seed_vault.clone()))
        .with(ShuttleDynSeed::new(objective_seed_vault.clone()))
        .with(DockyardDynSeed::new(
            objective_seed_vault.clone(),
            process_token_context.clone(),
        ))
        .with(AssemblerDynSeed::new(
            factory_seed_vault,
            objective_seed_vault,
            process_token_context,
        ))
        .with(CargoContainerDynSeed)
}
