use dudes_in_space_api::finance::BankRegistry;
use dudes_in_space_api::item::ItemVault;
use dudes_in_space_api::module::{Module, ProcessTokenContext};
use dudes_in_space_api::person::{ObjectiveDeciderVault, SubordinationTable};
use dudes_in_space_api::trade::OrderHolder;
use dudes_in_space_api::utils::request::ReqContext;
use dyn_serde::DynDeserializeSeedVault;
use std::rc::Rc;

pub struct Components {
    pub process_token_context: Rc<ProcessTokenContext>,
    pub req_context: Rc<ReqContext>,
    pub objectives_decider_vault: ObjectiveDeciderVault,
    pub item_vault: Rc<ItemVault>,
    pub order_holder: Rc<OrderHolder>,
    pub module_seed_vault: Rc<DynDeserializeSeedVault<dyn Module>>,
    pub subordination_table: Rc<SubordinationTable>,
    pub bank_registry: Rc<BankRegistry>,
}

pub fn core_components() -> Components {
    let process_token_context = Rc::new(ProcessTokenContext::new());
    let req_context = Rc::new(ReqContext::new());
    let item_vault = Rc::new(crate::register_items(ItemVault::new()));
    let order_holder = Rc::new(OrderHolder::new());
    let subordination_table = Rc::new(SubordinationTable::new());
    let bank_registry = Rc::new(BankRegistry::new());

    let objectives_seed_vault =
        crate::register_objectives(Default::default(), req_context.clone());
    let objectives_decider_vault =
        crate::register_objective_deciders(Default::default());

    let module_factory_seed_vault =
        crate::register_module_factories(Default::default()).into_rc();

    let module_seed_vault = crate::register_modules(
        Default::default(),
        module_factory_seed_vault,
        objectives_seed_vault.into_rc(),
        bank_registry.clone(),
        item_vault.clone(),
        order_holder.clone(),
        process_token_context.clone(),
    )
    .into_rc();

    Components {
        process_token_context,
        req_context,
        objectives_decider_vault,
        item_vault,
        order_holder,
        module_seed_vault,
        subordination_table,
        bank_registry,
    }
}
