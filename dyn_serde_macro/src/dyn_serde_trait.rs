use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;
use syn::{self, parse::Parse, parse_macro_input};

#[derive(Debug)]
struct ParsedInput {
    trait_name: Ident,
}

impl Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            panic!("Usage: dyn_serialize_trait!(MyTrait)");
        }
        Ok(Self {
            trait_name: input.parse::<Ident>()?,
        })
    }
}

pub(crate) fn dyn_serde_trait_impl(input: TokenStream) -> TokenStream {
    let input: ParsedInput = parse_macro_input!(input);
    let trait_name = input.trait_name;
    let seed_name = format_ident!("{}Seed", trait_name);

    quote! {
        impl serde::Serialize for dyn #trait_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                use std::ops::Deref as _;
                dyn_serde::dyn_serialize::<S, dyn #trait_name>(serializer, self.deref())
            }
        }

        #[derive(Clone)]
        pub(crate) struct #seed_name<'r> {
            reg: &'r dyn_serde::DynDeserializeSeedVault<dyn #trait_name>,
        }

        impl<'r> #seed_name<'r> {
            pub(crate) fn new(reg: &'r dyn_serde::DynDeserializeSeedVault<dyn #trait_name>) ->Self {
                Self { reg }
            }
        }

        impl<'b, 'de> serde::de::DeserializeSeed<'de> for #seed_name<'b> {
            type Value = Box<dyn #trait_name>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                self.reg.deserialize(deserializer)
            }
        }
    }
    .into()
}
