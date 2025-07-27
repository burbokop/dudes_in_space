pub(crate) const CORE_PACKAGE_ID: &str = "core";

mod modules;
mod objectives;

pub use modules::register_module_factories;
pub use modules::register_modules;
pub use objectives::register_objective_deciders;
pub use objectives::register_objectives;
