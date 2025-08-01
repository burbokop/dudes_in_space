#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(deprecated)]

pub(crate) const CORE_PACKAGE_ID: &str = "core";

pub mod env_presets;
mod modules;
mod objectives;

pub use modules::register_module_factories;
pub use modules::register_modules;
pub use objectives::register_objective_deciders;
pub use objectives::register_objectives;
