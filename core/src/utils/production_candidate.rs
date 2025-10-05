use dudes_in_space_api::environment::FindBestOffersForItemsResult;
use dudes_in_space_api::finance::{BankRegistry, Money, PossiblyNegativeMoney};
use dudes_in_space_api::item::ItemId;
use dudes_in_space_api::recipe::{ItemRecipe, OutputItemRecipe};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProductionCandidateEstimate {
    /// Price of all ingredients needed to produce do one crafting cycle.
    /// For example, if the recipe is
    /// ```
    /// use dudes_in_space_api::recipe::ItemRecipe;
    /// ItemRecipe {
    ///     input: [("biba".into(), 10)].into(),
    ///     output: [("boba".into(), 10)].into(),
    /// };
    /// ```
    /// then the field contains a cost price of 10 units of boba
    pub(crate) cost_price: Money,
    pub(crate) profit: PossiblyNegativeMoney,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProductionFromIngredientsCandidate {
    pub(crate) product: ItemId,
    pub(crate) average_product_sell_price: Money,
    pub(crate) average_ingredients_buy_price: BTreeMap<ItemId, Money>,
    pub(crate) has_all_ingredients_on_market: bool,
    pub(crate) has_producers_on_market: bool,
    pub(crate) recipe: ItemRecipe,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    pub(crate) estimate: Option<ProductionCandidateEstimate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProductionFromEnvironmentCandidate {
    pub(crate) product: ItemId,
    pub(crate) average_product_sell_price: Money,
    pub(crate) has_producers_on_market: bool,
    pub(crate) recipe: OutputItemRecipe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "tp")]
pub(crate) enum ProductionCandidate {
    FromIngredients(ProductionFromIngredientsCandidate),
    FromEnvironment(ProductionFromEnvironmentCandidate),
}

impl ProductionCandidate {
    fn product(&self) -> &ItemId {
        match &self {
            ProductionCandidate::FromIngredients(c) => &c.product,
            ProductionCandidate::FromEnvironment(c) => &c.product,
        }
    }

    fn has_producers_on_market(&self) -> bool {
        match &self {
            Self::FromIngredients(c) => c.has_producers_on_market,
            Self::FromEnvironment(c) => c.has_producers_on_market,
        }
    }
}

impl ProductionCandidate {
    pub(crate) fn build_for_items_in_demand(
        search_result: &FindBestOffersForItemsResult,
        recipes_to_consider: &BTreeSet<ItemRecipe>,
        output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
        bank_registry: &BankRegistry,
    ) -> Vec<Self> {
        Self::build_for_items(
            search_result,
            recipes_to_consider,
            output_recipes_to_consider,
            bank_registry,
            collect_items_in_demand(&search_result, &output_recipes_to_consider),
        )
    }

    pub(crate) fn build_for_items(
        search_result: &FindBestOffersForItemsResult,
        recipes_to_consider: &BTreeSet<ItemRecipe>,
        output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
        bank_registry: &BankRegistry,
        items: BTreeMap<ItemId, Money>,
    ) -> Vec<Self> {
        items
            .into_iter()
            .filter_map(|(item, _)| {
                let has_producers_on_market = search_result
                    .average_buy_offers
                    .iter()
                    .find(|(offer_item, _)| *offer_item == &item)
                    .is_some();

                match recipes_to_consider.iter().find(|recipe| {
                    recipe
                        .output
                        .items()
                        .find(|recipe_item| *recipe_item == &item)
                        .is_some()
                }) {
                    None => {
                        match output_recipes_to_consider.iter().find(|recipe| {
                            recipe
                                .items()
                                .find(|recipe_item| *recipe_item == &item)
                                .is_some()
                        }) {
                            None => {
                                todo!()
                            }
                            Some(recipe) => {
                                // TODO: Should check if the output recipe requires some action to produce
                                // if not return true
                                // if yes, check if it can do this action (for example, mine asteroids or collect gas from nebula, or some other natural occurring resource)

                                if let Some((_, average_product_sell_price)) = search_result
                                    .average_sell_offers
                                    .iter()
                                    .find(|(offer_item, _)| *offer_item == &item)
                                {
                                    Some(Self::FromEnvironment(
                                        ProductionFromEnvironmentCandidate {
                                            product: item.clone(),
                                            average_product_sell_price: average_product_sell_price
                                                .clone(),
                                            has_producers_on_market,
                                            recipe: recipe.clone(),
                                        },
                                    ))
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    Some(recipe) => {
                        assert!(!recipe.input.is_empty());

                        let mut has_all_ingredients_on_market: bool = true;
                        let mut average_ingredients_buy_price: BTreeMap<ItemId, Money> =
                            Default::default();
                        let mut sum_ingredients_cost_price: Option<Money> = None;
                        for (item, count) in &recipe.input {
                            if let Some((_, average_buy_price)) = search_result
                                .average_buy_offers
                                .iter()
                                .find(|(offer_item, _)| *offer_item == item)
                            {
                                average_ingredients_buy_price
                                    .try_insert(item.clone(), average_buy_price.clone())
                                    .unwrap();

                                let stack_price = average_buy_price.clone() * *count;
                                match &mut sum_ingredients_cost_price {
                                    None => sum_ingredients_cost_price = Some(stack_price),
                                    Some(sum_ingredients_cost_price) => sum_ingredients_cost_price
                                        .add_assign_same_currency(stack_price)
                                        .unwrap(),
                                }
                            } else {
                                has_all_ingredients_on_market = false;
                            }
                        }

                        if let Some((_, average_product_sell_price)) = search_result
                            .average_sell_offers
                            .iter()
                            .find(|(offer_item, _)| *offer_item == &item)
                        {
                            Some(Self::FromIngredients(ProductionFromIngredientsCandidate {
                                product: item.clone(),
                                average_product_sell_price: average_product_sell_price.clone(),
                                average_ingredients_buy_price,
                                has_all_ingredients_on_market,
                                has_producers_on_market,
                                recipe: recipe.clone(),
                                estimate: sum_ingredients_cost_price.map(
                                    |sum_ingredients_cost_price| ProductionCandidateEstimate {
                                        cost_price: sum_ingredients_cost_price.clone(),
                                        profit: (average_product_sell_price.clone()
                                            * recipe.output.count(&item).unwrap())
                                        .sub(bank_registry, sum_ingredients_cost_price),
                                    },
                                ),
                            }))
                        } else {
                            None
                        }
                    }
                }
            })
            .collect()
    }
}

pub(crate) trait SortProductionCandidate {
    /// Sorts in a way so that more preferred candidates are at the front.
    /// Prefers candidates with more profitability, less competition, more complex recipes, better availability of ingredients.
    fn sort(&mut self, bank_registry: &BankRegistry, preferred_product: Option<ItemId>);
}

impl SortProductionCandidate for Vec<ProductionCandidate> {
    fn sort(&mut self, bank_registry: &BankRegistry, preferred_product: Option<ItemId>) {
        self.sort_by(|a, b| {
            preferred_product
                .as_ref()
                .map(|preferred_product| {
                    (*a.product() == *preferred_product)
                        .cmp(&(*b.product() == *preferred_product))
                        .reverse()
                })
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    a.has_producers_on_market()
                        .cmp(&b.has_producers_on_market())
                        .then_with(|| match (a, b) {
                            (
                                ProductionCandidate::FromIngredients(a),
                                ProductionCandidate::FromIngredients(b),
                            ) => a
                                .has_all_ingredients_on_market
                                .cmp(&b.has_all_ingredients_on_market)
                                .reverse()
                                .then_with(|| {
                                    cmp_option(a.estimate.as_ref(), b.estimate.as_ref(), |a, b| {
                                        a.profit.cmp(bank_registry, &b.profit).reverse().then_with(
                                            || a.cost_price.cmp(bank_registry, &b.cost_price),
                                        )
                                    })
                                }),
                            (
                                ProductionCandidate::FromEnvironment(a),
                                ProductionCandidate::FromEnvironment(b),
                            ) => Ordering::Equal,
                            (
                                ProductionCandidate::FromIngredients(a),
                                ProductionCandidate::FromEnvironment(b),
                            ) => Ordering::Less,
                            (
                                ProductionCandidate::FromEnvironment(a),
                                ProductionCandidate::FromIngredients(b),
                            ) => Ordering::Greater,
                        })
                })
        });
    }
}

fn collect_items_in_demand(
    search_result: &FindBestOffersForItemsResult,
    output_recipes_to_consider: &BTreeSet<OutputItemRecipe>,
) -> BTreeMap<ItemId, Money> {
    search_result
        .average_sell_offers
        .clone()
        .into_iter()
        .filter(|(item, _)| {
            output_recipes_to_consider
                .iter()
                .find(|recipe| {
                    recipe
                        .items()
                        .find(|recipe_item| *recipe_item == item)
                        .is_some()
                })
                .is_some()
        })
        .collect()
}

fn cmp_option<T, F: FnOnce(T, T) -> Ordering>(a: Option<T>, b: Option<T>, f: F) -> Ordering {
    let a_is_none = a.is_none();
    if let (Some(a), Some(b)) = (a, b) {
        f(a, b)
    } else if a_is_none {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

impl Display for ProductionCandidateEstimate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "estimate(c: {}, p: {})", self.cost_price, self.profit,)
    }
}

impl Display for ProductionFromIngredientsCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.estimate {
            None => write!(
                f,
                "p: {}, avr_prod_sell_price: {}, has_all_ing_on_m: {}, has_producers_on_m: {}, r: {}",
                self.product,
                self.average_product_sell_price,
                self.has_all_ingredients_on_market,
                self.has_producers_on_market,
                self.recipe,
            ),
            Some(estimate) => write!(
                f,
                "p: {}, avr_prod_sell_price: {}, has_all_ing_on_m: {}, has_producers_on_m: {}, r: {}, est: {}",
                self.product,
                self.average_product_sell_price,
                self.has_all_ingredients_on_market,
                self.recipe,
                self.has_producers_on_market,
                estimate,
            ),
        }
    }
}

impl Display for ProductionFromEnvironmentCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "p: {}, avr_prod_sell_price: {}, has_producers_on_m: {}, r: {}",
            self.product,
            self.average_product_sell_price,
            self.has_producers_on_market,
            self.recipe,
        )
    }
}

impl Display for ProductionCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FromIngredients(c) => c.fmt(f),
            Self::FromEnvironment(c) => c.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dudes_in_space_api::{money, possibly_negative_money};

    #[test]
    fn sort_by_preferred_product_test() {
        let bank_registry = BankRegistry::new();

        let mut production_candidates = vec![
            ProductionCandidate::FromEnvironment(ProductionFromEnvironmentCandidate {
                product: "biba".to_string(),
                average_product_sell_price: money!(100, "USD"),
                has_producers_on_market: true,
                recipe: [("biba".into(), 10)].into(),
            }),
            ProductionCandidate::FromEnvironment(ProductionFromEnvironmentCandidate {
                product: "boba".to_string(),
                average_product_sell_price: money!(100, "USD"),
                has_producers_on_market: true,
                recipe: [("boba".into(), 10)].into(),
            }),
        ];

        production_candidates.sort(&bank_registry, Some("boba".into()));

        assert_eq!(production_candidates[0].product().as_str(), "boba");
        assert_eq!(production_candidates[1].product().as_str(), "biba");
    }

    /// Candidates that don't have producers on the market yet are preferred because of zero competition.
    #[test]
    fn sort_by_has_producers_on_market_test() {
        let bank_registry = BankRegistry::new();

        let mut production_candidates = vec![
            ProductionCandidate::FromEnvironment(ProductionFromEnvironmentCandidate {
                product: "biba".to_string(),
                average_product_sell_price: money!(100, "USD"),
                has_producers_on_market: true,
                recipe: [("biba".into(), 10)].into(),
            }),
            ProductionCandidate::FromEnvironment(ProductionFromEnvironmentCandidate {
                product: "boba".to_string(),
                average_product_sell_price: money!(100, "USD"),
                has_producers_on_market: false,
                recipe: [("boba".into(), 10)].into(),
            }),
        ];

        production_candidates.sort(&bank_registry, None);

        assert_eq!(production_candidates[0].product().as_str(), "boba");
        assert_eq!(production_candidates[1].product().as_str(), "biba");
    }

    #[test]
    fn sort_by_enum_test() {
        let bank_registry = BankRegistry::new();

        let mut production_candidates = vec![
            ProductionCandidate::FromEnvironment(ProductionFromEnvironmentCandidate {
                product: "biba".to_string(),
                average_product_sell_price: money!(100, "USD"),
                has_producers_on_market: true,
                recipe: [("biba".into(), 10)].into(),
            }),
            ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                product: "boba".to_string(),
                average_product_sell_price: money!(100, "USD"),
                average_ingredients_buy_price: Default::default(),
                has_all_ingredients_on_market: false,
                has_producers_on_market: true,
                recipe: ItemRecipe {
                    input: [("biba".into(), 10)].into(),
                    output: [("boba".into(), 10)].into(),
                },
                estimate: None,
            }),
        ];

        production_candidates.sort(&bank_registry, None);

        assert_eq!(production_candidates[0].product().as_str(), "boba");
        assert_eq!(production_candidates[1].product().as_str(), "biba");
    }

    #[test]
    fn sort_by_has_all_ingredients_on_market_test() {
        let bank_registry = BankRegistry::new();

        {
            let mut production_candidates = vec![
                ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                    product: "biba".to_string(),
                    average_product_sell_price: money!(100, "USD"),
                    average_ingredients_buy_price: [("pipa".into(), money!(10, "USD"))].into(),
                    has_all_ingredients_on_market: false,
                    has_producers_on_market: true,
                    recipe: ItemRecipe {
                        input: [("pipa".into(), 10)].into(),
                        output: [("biba".into(), 10)].into(),
                    },
                    estimate: None,
                }),
                ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                    product: "boba".to_string(),
                    average_product_sell_price: money!(100, "USD"),
                    average_ingredients_buy_price: [("biba".into(), money!(10, "USD"))].into(),
                    has_all_ingredients_on_market: true,
                    has_producers_on_market: true,
                    recipe: ItemRecipe {
                        input: [("biba".into(), 10)].into(),
                        output: [("boba".into(), 10)].into(),
                    },
                    estimate: None,
                }),
            ];

            production_candidates.sort(&bank_registry, None);

            assert_eq!(production_candidates[0].product().as_str(), "boba");
            assert_eq!(production_candidates[1].product().as_str(), "biba");
        }
    }

    #[test]
    fn sort_by_profit_test() {
        let bank_registry = BankRegistry::new();

        let mut production_candidates = vec![
            ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                product: "biba".to_string(),
                average_product_sell_price: money!(200, "USD"),
                average_ingredients_buy_price: [("pipa".into(), money!(100, "USD"))].into(),
                has_all_ingredients_on_market: true,
                has_producers_on_market: true,
                recipe: ItemRecipe {
                    input: [("pipa".into(), 10)].into(),
                    output: [("biba".into(), 10)].into(),
                },
                estimate: Some(ProductionCandidateEstimate {
                    cost_price: money!(1000, "USD"),
                    profit: possibly_negative_money!(1000, "USD"),
                }),
            }),
            ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                product: "boba".to_string(),
                average_product_sell_price: money!(300, "USD"),
                average_ingredients_buy_price: [("biba".into(), money!(150, "USD"))].into(),
                has_all_ingredients_on_market: true,
                has_producers_on_market: true,
                recipe: ItemRecipe {
                    input: [("biba".into(), 10)].into(),
                    output: [("boba".into(), 10)].into(),
                },
                estimate: Some(ProductionCandidateEstimate {
                    cost_price: money!(1500, "USD"),
                    profit: possibly_negative_money!(1500, "USD"),
                }),
            }),
        ];

        production_candidates.sort(&bank_registry, None);

        assert_eq!(production_candidates[0].product().as_str(), "boba");
        assert_eq!(production_candidates[1].product().as_str(), "biba");
    }

    #[test]
    fn sort_by_cost_price_test() {
        let bank_registry = BankRegistry::new();

        let mut production_candidates = vec![
            ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                product: "biba".to_string(),
                average_product_sell_price: money!(200, "USD"),
                average_ingredients_buy_price: [("pipa".into(), money!(100, "USD"))].into(),
                has_all_ingredients_on_market: true,
                has_producers_on_market: true,
                recipe: ItemRecipe {
                    input: [("pipa".into(), 10)].into(),
                    output: [("biba".into(), 10)].into(),
                },
                estimate: Some(ProductionCandidateEstimate {
                    cost_price: money!(1000, "USD"),
                    profit: possibly_negative_money!(1000, "USD"),
                }),
            }),
            ProductionCandidate::FromIngredients(ProductionFromIngredientsCandidate {
                product: "boba".to_string(),
                average_product_sell_price: money!(300, "USD"),
                average_ingredients_buy_price: [("biba".into(), money!(200, "USD"))].into(),
                has_all_ingredients_on_market: true,
                has_producers_on_market: true,
                recipe: ItemRecipe {
                    input: [("biba".into(), 10)].into(),
                    output: [("boba".into(), 10)].into(),
                },
                estimate: Some(ProductionCandidateEstimate {
                    cost_price: money!(2000, "USD"),
                    profit: possibly_negative_money!(1000, "USD"),
                }),
            }),
        ];

        production_candidates.sort(&bank_registry, None);

        assert_eq!(production_candidates[0].product().as_str(), "biba");
        assert_eq!(production_candidates[1].product().as_str(), "boba");
    }
}
