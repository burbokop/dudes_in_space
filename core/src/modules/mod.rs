mod assembler;
mod core_module;
mod dockyard;
mod personnel_area;
mod shuttle;

pub use assembler::*;
pub use core_module::*;
pub use dockyard::*;
use dudes_in_space_api::module::{Module, ProcessTokenContext};
use dudes_in_space_api::recipe::ModuleFactory;
use dyn_serde::DynDeserializeSeedVault;
pub use personnel_area::*;
pub use shuttle::*;
use std::rc::Rc;

pub fn register_module_factories(
    vault: DynDeserializeSeedVault<dyn ModuleFactory>,
) -> DynDeserializeSeedVault<dyn ModuleFactory> {
    vault
        .with(ShuttleFactoryDynSeed)
        .with(DockyardFactoryDynSeed)
}

pub fn register_modules(
    vault: DynDeserializeSeedVault<dyn Module>,
    factory_seed_vault: Rc<DynDeserializeSeedVault<dyn ModuleFactory>>,
    process_token_context: Rc<ProcessTokenContext>,
) -> DynDeserializeSeedVault<dyn Module> {
    vault
        .with(PersonnelAreaDynSeed)
        .with(ShuttleDynSeed)
        .with(DockyardDynSeed::new(process_token_context.clone()))
        .with(AssemblerDynSeed::new(
            factory_seed_vault,
            process_token_context,
        ))
}
