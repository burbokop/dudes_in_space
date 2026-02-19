use crate::person::PurchasedItemsMaxPrices;
use crate::utils::utils::Float;
use burbomath::math::NonNeg;
use serde::{Deserialize, Serialize};
use serde_intermediate::Intermediate;
use std::collections::BTreeMap;

static DEFAULT_MARGIN: Float = 2.0;
static MIN_MARGIN: Float = 1.05;
static MARGIN_CHANGE: Float = 1.05;

#[derive(Serialize, Deserialize, Debug)]
pub struct Blueprint {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonalNotes {
    margin: Float,
    blueprints: Vec<Blueprint>,
    #[serde(default)]
    purchased_items_max_prices: PurchasedItemsMaxPrices,
    /// For mods
    custom_data: BTreeMap<String, Intermediate>,
}

impl PersonalNotes {
    pub fn blueprints(&self) -> &[Blueprint] {
        &self.blueprints
    }

    pub fn margin(&self) -> NonNeg<Float> {
        NonNeg::new(self.margin).unwrap()
    }

    pub fn custom_data(&self) -> &BTreeMap<String, Intermediate> {
        &self.custom_data
    }

    pub fn custom_data_mut(&mut self) -> &mut BTreeMap<String, Intermediate> {
        &mut self.custom_data
    }

    pub fn purchased_items_max_prices(&self) -> &PurchasedItemsMaxPrices {
        &self.purchased_items_max_prices
    }

    pub fn purchased_items_max_prices_mut(&mut self) -> &mut PurchasedItemsMaxPrices {
        &mut self.purchased_items_max_prices
    }

    pub fn increase_margin(&mut self) -> bool {
        self.margin *= MARGIN_CHANGE;
        if self.margin < MIN_MARGIN {
            self.margin = MIN_MARGIN;
            false
        } else {
            true
        }
    }

    pub fn decrease_margin(&mut self) -> bool {
        self.margin /= MARGIN_CHANGE;
        if self.margin < MIN_MARGIN {
            self.margin = MIN_MARGIN;
            false
        } else {
            true
        }
    }
}

impl Default for PersonalNotes {
    fn default() -> Self {
        Self {
            margin: DEFAULT_MARGIN,
            blueprints: Vec::new(),
            purchased_items_max_prices: Default::default(),
            custom_data: BTreeMap::new(),
        }
    }
}
