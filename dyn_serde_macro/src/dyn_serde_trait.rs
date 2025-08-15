use proc_macro::TokenStream;
use quote::quote;
use syn::{self};
use syn::{Expr, Ident};

#[derive(deluxe::ParseMetaItem, Debug)]
#[deluxe(attributes(serde))]
struct Input(Expr, Ident);

pub(crate) fn dyn_serde_trait_impl(input: TokenStream) -> TokenStream {
    let input: Input = deluxe::parse(input).expect("Wrong macro input.");
    let Input(trait_name, seed_name) = input;

    // let sync_seed_name = Ident::new(&format!("Sync{}", seed_name), seed_name.span());

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

        // impl serde::Serialize for dyn #trait_name + Sync {
        //     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        //     where
        //         S: serde::Serializer
        //     {
        //         use std::ops::Deref as _;
        //         dyn_serde::dyn_serialize::<S, dyn #trait_name>(serializer, self.deref())
        //     }
        // }

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

        // #[derive(Clone)]
        // pub(crate) struct #sync_seed_name<'r> {
        //     reg: &'r dyn_serde::DynDeserializeSeedVault<dyn #trait_name>,
        // }
        //
        // impl<'r> #sync_seed_name<'r> {
        //     pub(crate) fn new(reg: &'r dyn_serde::DynDeserializeSeedVault<dyn #trait_name>) ->Self {
        //         Self { reg }
        //     }
        // }
        //
        // impl<'b, 'de> serde::de::DeserializeSeed<'de> for #sync_seed_name<'b> {
        //     type Value = Box<dyn #trait_name + Sync>;
        //
        //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        //     where
        //         D: serde::de::Deserializer<'de>,
        //     {
        //         self.reg.deserialize_sync(deserializer)
        //     }
        // }
    }
    .into()
}
