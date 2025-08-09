#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(deprecated)]

pub(crate) const CORE_PACKAGE_ID: &str = "core";

pub mod env_presets;
mod modules;
mod objectives;

use dudes_in_space_api::item::{Item, ItemVault};
use dudes_in_space_api::utils::physics::{KgPerM3, M3};
pub use modules::register_module_factories;
pub use modules::register_modules;
pub use objectives::register_objective_deciders;
pub use objectives::register_objectives;

pub fn register_items(vault: ItemVault) -> ItemVault {
    vault.with(Item::new(
        "steel".into(),
        M3(1),
        KgPerM3::from(7850),
    )).with(Item::new(
        "plastic".into(),
        M3(1),
        KgPerM3::from(900),
    )).with(Item::new(
        "microelectronics".into(),
        M3(10),
        KgPerM3::from(2500),
    ))
}