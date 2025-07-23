pub(crate) const CORE_PACKAGE_ID: &str = "core";

pub mod modules;

pub use modules::register_module_factories;
pub use modules::register_modules;