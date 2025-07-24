mod assembler;
mod core_module;
mod dockyard;
mod personnel_area;
mod shuttle;

pub use assembler::*;
pub use core_module::*;
pub use dockyard::*;
use dudes_in_space_api::module::Module;
use dudes_in_space_api::recipe::ModuleFactory;
use dyn_serde::DynDeserializeSeedVault;
pub use personnel_area::*;
pub use shuttle::*;
use std::rc::Rc;

pub fn register_module_factories(
    vault: DynDeserializeSeedVault<dyn ModuleFactory>,
) -> DynDeserializeSeedVault<dyn ModuleFactory> {
    DynDeserializeSeedVault::<dyn ModuleFactory>::new()
        .with(ShuttleFactoryDynSeed)
        .with(DockyardFactoryDynSeed)
}

pub fn register_modules(
    vault: DynDeserializeSeedVault<dyn Module>,
    factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
) -> DynDeserializeSeedVault<dyn Module> {
    DynDeserializeSeedVault::<dyn Module>::new()
        .with(PersonnelAreaDynSeed)
        .with(ShuttleDynSeed)
        .with(DockyardDynSeed)
        .with(AssemblerDynSeed::new(factory_seed_vault))
}
