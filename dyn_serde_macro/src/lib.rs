#![feature(iter_intersperse)]

mod deserialize_seed;
mod dyn_serde_trait;

use crate::deserialize_seed::deserialize_seed_impl;
use crate::dyn_serde_trait::dyn_serde_trait_impl;
use proc_macro::TokenStream;

#[proc_macro]
pub fn dyn_serde_trait(input: TokenStream) -> TokenStream {
    dyn_serde_trait_impl(input)
}

#[proc_macro_derive(DeserializeSeedXXX, attributes(deserialize_seed_xxx))]
pub fn deserialize_seed(input: TokenStream) -> TokenStream {
    deserialize_seed_impl(input)
}
