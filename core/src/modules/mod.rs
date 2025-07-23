mod assembler;
mod core_module;
mod dockyard;
mod personnel_area;
mod shuttle;

use std::rc::Rc;
pub use assembler::*;
pub use core_module::*;
pub use dockyard::*;
pub use personnel_area::*;
pub use shuttle::*;

use dudes_in_space_api::modules::{Module, ModuleFactory};
use dyn_serde::DynDeserializeSeedVault;

pub fn register_module_factories(vault: DynDeserializeSeedVault<dyn ModuleFactory>) -> DynDeserializeSeedVault<dyn ModuleFactory> {
    DynDeserializeSeedVault::<dyn ModuleFactory>::new()
        .with(ShuttleFactoryDynSeed)
        .with(DockyardFactoryDynSeed)
}

pub fn register_modules(vault: DynDeserializeSeedVault<dyn Module>, factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>) -> DynDeserializeSeedVault<dyn Module> {
    DynDeserializeSeedVault::<dyn Module>::new()
        .with(PersonnelAreaDynSeed)
        .with(ShuttleDynSeed)
        .with(DockyardDynSeed)
        .with(AssemblerDynSeed::new(factory_seed_vault))
}
