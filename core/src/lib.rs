#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(deprecated)]

pub(crate) const CORE_PACKAGE_ID: &str = "core";

pub mod env_presets;
mod modules;
mod objectives;

use dudes_in_space_api::item::{Item, ItemVault};
use dudes_in_space_api::recipe::ItemRecipe;
use dudes_in_space_api::utils::physics::{KgPerM3, M3};
pub use modules::register_module_factories;
pub use modules::register_modules;
pub use objectives::register_objective_deciders;
pub use objectives::register_objectives;
use std::sync::LazyLock;

pub fn register_items(vault: ItemVault) -> ItemVault {
    vault
        .with(Item::new("gangue".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new("biomass".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new("silicon_ore".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new("iron_ore".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(
            "rare_earth_ore".into(),
            M3(1),
            KgPerM3::from(7850),
        ))
        .with(Item::new("ice".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new("water".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new("carbon".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new("plastic".into(), M3(1), KgPerM3::from(900)))
        .with(Item::new("silicon".into(), M3(1), KgPerM3::from(900)))
        .with(Item::new("steel".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(
            "rare_earth_alloys".into(),
            M3(1),
            KgPerM3::from(7850),
        ))
        .with(Item::new("heat_cell".into(), M3(1), KgPerM3::from(7850)))
        .with(Item::new(
            "hot_heat_cell".into(),
            M3(1),
            KgPerM3::from(7850),
        ))
        .with(Item::new(
            "microelectronics".into(),
            M3(10),
            KgPerM3::from(2500),
        ))
}

/// basic resources:
///    biomass
///    silicon_ore
///    iron_ore
///    rare_earth_ore
///    ice
pub(crate) static RECIPES: LazyLock<[ItemRecipe; 10]> = LazyLock::new(|| {
    [
        ItemRecipe {
            input: [("ice".into(), 10)].into(),
            output: [("water".into(), 10), ("gangue".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("biomass".into(), 10)].into(),
            output: [("carbon".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("carbon".into(), 10)].into(),
            output: [("plastic".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("silicon_ore".into(), 10)].into(),
            output: [("silicon".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("iron_ore".into(), 10), ("carbon".into(), 10)].into(),
            output: [("steel".into(), 10), ("gangue".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("rare_earth_ore".into(), 10)].into(),
            output: [("rare_earth_alloys".into(), 10), ("gangue".into(), 10)].into(),
        },
        ItemRecipe {
            input: [
                ("silicon".into(), 10),
                ("rare_earth_alloys".into(), 10),
                ("plastic".into(), 10),
            ]
            .into(),
            output: [("microelectronics".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("steel".into(), 10)].into(),
            output: [("heat_cell".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("steel".into(), 10)].into(),
            output: [("heat_cell".into(), 10)].into(),
        },
        ItemRecipe {
            input: [("steel".into(), 10)].into(),
            output: [("heat_cell".into(), 10)].into(),
        },
    ]
});

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
