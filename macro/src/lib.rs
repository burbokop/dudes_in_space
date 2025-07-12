use proc_macro::TokenStream;
use syn::{self, parse::Parse, parse_macro_input, Attribute, DeriveInput};
use quote::{format_ident, quote, ToTokens};
use syn::token::Trait;
use syn::{Ident, LitInt, Token};

#[derive(Debug)]
struct ParsedInput {
    trait_name: Ident,
}

impl Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            panic!("Usage: dyn_serialize_trait!(MyTrait)");
        }
        Ok(Self { trait_name: input.parse::<Ident>()? })
    }
}

#[proc_macro]
pub fn dyn_serialize_trait(input: TokenStream) -> TokenStream {
    let input: ParsedInput = parse_macro_input!(input);
    let trait_name = input.trait_name;
    let seed_name = format_ident!("{}Seed", trait_name);

    quote! {
        impl Serialize for Box<dyn #trait_name> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer
            {
                use std::ops::Deref as _;
                crate::bl::utils::dyn_serde::dyn_serialize::<S, dyn #trait_name>(serializer, self.deref())
            }
        }

        pub(crate) struct #seed_name<'r> {
            reg: &'r crate::bl::utils::dyn_serde::DynDeserializeFactoryRegistry<dyn #trait_name>,
        }

        impl<'r> #seed_name<'r> {
            pub(crate) fn new(reg: &'r crate::bl::utils::dyn_serde::DynDeserializeFactoryRegistry<dyn #trait_name>) ->Self {
                Self { reg }
            }
        }

        impl<'b, 'de> serde::de::DeserializeSeed<'de> for #seed_name<'b> {
            type Value = Box<dyn #trait_name>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                self.reg.deserialize(deserializer)
            }
        }
    }
    .into()
}
