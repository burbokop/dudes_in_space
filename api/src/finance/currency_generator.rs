use crate::finance::BankRegistry;
use crate::person::PersonInfo;
use rand::Rng;
use rand::seq::IndexedRandom;
use std::collections::BTreeSet;

static NAMES: &[&str] = &[
    "Meme coin",
    "Dogecoin",
    "Burbokoin",
    "Credits",
    "Latinum",
    "$",
    "€",
    "₴",
    "Tax-free minimum",
];

pub struct CurrencyGenerator {}

impl CurrencyGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_name<R: Rng>(
        &self,
        rng: &mut R,
        bank_registry: &BankRegistry,
        person_info: &PersonInfo,
    ) -> String {
        let mut names: BTreeSet<_> = NAMES.iter().map(|x| x.to_string()).collect();
        loop {
            let potential_name = NAMES.choose(rng).unwrap().to_string();
            if !bank_registry.contains_currency(&potential_name) {
                break potential_name;
            }
            names.remove(&potential_name);
        }
    }
}
