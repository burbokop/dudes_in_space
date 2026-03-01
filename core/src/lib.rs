#![feature(map_try_insert)]
// #![feature(push_mut)]
#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(deprecated)]

pub(crate) const CORE_PACKAGE_ID: &str = "core";

pub mod components;
pub mod env_presets;
mod modules;
mod objectives;
mod utils;

use burbomath::physics::{KgPerM3, M3};
use dudes_in_space_api::item::{Item, ItemVault};
use dudes_in_space_api::recipe::ItemRecipe;
pub use modules::register_module_factories;
pub use modules::register_modules;
pub use modules::types as module_types;
pub use objectives::register_objective_deciders;
pub use objectives::register_objectives;
use std::sync::LazyLock;

// TODO: remove this (or not?)
pub mod __modules {
    pub use super::modules::Assembler;
}

pub mod items {
    pub static GANGUE: &str = "gangue";
    pub static BIOMASS: &str = "biomass";
    pub static SILICON_ORE: &str = "silicon_ore";
    pub static IRON_ORE: &str = "iron_ore";
    pub static RARE_EARTH_ORE: &str = "rare_earth_ore";
    pub static ICE: &str = "ice";
    pub static WATER: &str = "water";
    pub static CARBON: &str = "carbon";
    pub static PLASTIC: &str = "plastic";
    pub static SILICON: &str = "silicon";
    pub static STEEL: &str = "steel";
    pub static RARE_EARTH_ALLOYS: &str = "rare_earth_alloys";
    pub static HEAT_CELL: &str = "heat_cell";
    pub static HOT_HEAT_CELL: &str = "hot_heat_cell";
    pub static MICROELECTRONICS: &str = "microelectronics";
}

pub fn register_items(vault: ItemVault) -> ItemVault {
    use items::*;
    vault
        .with(Item::new(GANGUE.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(BIOMASS.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(SILICON_ORE.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(IRON_ORE.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(RARE_EARTH_ORE.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(ICE.into(), M3(1), KgPerM3::from(917)))
        .with(Item::new(WATER.into(), M3(1), KgPerM3::from(1000)))
        .with(Item::new(CARBON.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(PLASTIC.into(), M3(1), KgPerM3::from(900)))
        .with(Item::new(SILICON.into(), M3(1), KgPerM3::from(2329)))
        .with(Item::new(STEEL.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(
            RARE_EARTH_ALLOYS.into(),
            M3(1),
            KgPerM3::from(7850),
        ))
        .with(Item::new(HEAT_CELL.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(HOT_HEAT_CELL.into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(
            MICROELECTRONICS.into(),
            M3(10),
            KgPerM3::from(2500),
        ))
}

pub(crate) static HEAT_EXCHANGE_RECIPES: LazyLock<[ItemRecipe; 2]> = LazyLock::new(|| {
    [
        ItemRecipe {
            input: [("heat_cell".into(), 1)].into(),
            output: [("hot_heat_cell".into(), 1)].into(),
        },
        ItemRecipe {
            input: [("hot_heat_cell".into(), 1)].into(),
            output: [("heat_cell".into(), 1)].into(),
        },
    ]
});
