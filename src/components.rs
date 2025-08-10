use dudes_in_space_api::item::ItemVault;
use dudes_in_space_api::module::{Module, ProcessTokenContext};
use dudes_in_space_api::person::ObjectiveDeciderVault;
use dudes_in_space_api::utils::request::ReqContext;
use dyn_serde::DynDeserializeSeedVault;
use std::rc::Rc;

pub(crate) struct Components {
    pub(crate) process_token_context: Rc<ProcessTokenContext>,
    pub(crate) req_context: Rc<ReqContext>,
    pub(crate) objectives_decider_vault: ObjectiveDeciderVault,
    pub(crate) item_vault: Rc<ItemVault>,
    pub(crate) module_seed_vault: Rc<DynDeserializeSeedVault<dyn Module>>,
}

pub(crate) fn core_components() -> Components {
    let process_token_context = Rc::new(ProcessTokenContext::new());
    let req_context = Rc::new(ReqContext::new());

    let item_vault = Rc::new(dudes_in_space_core::register_items(ItemVault::new()));

    let objectives_seed_vault =
        dudes_in_space_core::register_objectives(Default::default(), req_context.clone());
    let objectives_decider_vault =
        dudes_in_space_core::register_objective_deciders(Default::default());

    let module_factory_seed_vault =
        dudes_in_space_core::register_module_factories(Default::default()).into_rc();

    let module_seed_vault = dudes_in_space_core::register_modules(
        Default::default(),
        module_factory_seed_vault,
        objectives_seed_vault.into_rc(),
        item_vault.clone(),
        process_token_context.clone(),
    )
    .into_rc();

    Components {
        process_token_context,
        req_context,
        objectives_decider_vault,
        item_vault,
        module_seed_vault,
    }
}
