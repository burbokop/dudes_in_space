use crate::environment::Cycle;
use crate::finance::{BankRegistry, Money};
use crate::item::ItemId;
use crate::recipe::InputItemRecipe;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PurchasedItemsMaxPrices {
    fresh: BTreeMap<ItemId, Money>,
    stabilized: BTreeMap<ItemId, Money>,
    cycles_to_next_stabilization: Cycle,
}

impl PurchasedItemsMaxPrices {
    pub fn stabilized(&self) -> &BTreeMap<ItemId, Money> {
        &self.stabilized
    }

    pub fn fresh(&self) -> &BTreeMap<ItemId, Money> {
        &self.fresh
    }

    pub fn stabilize(&mut self) {
        for (item_id, max_price) in self.fresh.iter() {
            self.stabilized.insert(item_id.clone(), max_price.clone());
        }
        self.fresh.clear();
    }

    pub fn register_purchase(
        &mut self,
        bank_registry: &BankRegistry,
        item_id: ItemId,
        price: Money,
    ) {
        let _ = self.fresh.try_insert(item_id.clone(), price.clone());

        if let Some(current_price) = self.fresh.get(&item_id) {
            self.fresh
                .insert(item_id, current_price.clone().max(bank_registry, price));
        } else {
            self.fresh.insert(item_id, price);
        }
    }

    pub(crate) fn proceed(&mut self) {
        if self.cycles_to_next_stabilization == 0 {
            self.stabilize();
            self.cycles_to_next_stabilization = 64;
        } else {
            self.cycles_to_next_stabilization -= 1;
        }
    }

    pub fn calculate_cost_price(
        &self,
        input: InputItemRecipe,
    ) -> Result<Money, CalculateCostPriceError> {
        // maybe should be `Money::sum_as`
        Money::try_sum_same_currency(input.into_iter().map(|stack| {
            if let Some(price) = self.stabilized().get(&stack.id) {
                Ok(price.clone() * stack.count)
            } else {
                Err(CalculateCostPriceError::ItemIsNotPurchased { item: stack.id })
            }
        }))
        .and_then(|x| x.ok_or(CalculateCostPriceError::EmptyInput))
    }
}

#[derive(Debug)]
pub enum CalculateCostPriceError {
    EmptyInput,
    ItemIsNotPurchased { item: ItemId },
}

impl Display for CalculateCostPriceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CalculateCostPriceError {}
