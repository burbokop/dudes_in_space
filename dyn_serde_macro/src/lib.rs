#![feature(iter_intersperse)]

use convert_case::{Case, Casing};
use darling::util::PathList;
use darling::{FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro::TokenStream;
use std::fmt::format;
use std::marker::PhantomData;
use quote::{ToTokens, format_ident, quote};
use std::ops::Deref;
use syn::ReturnType::Default;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::token::Trait;
use syn::{
    self, Attribute, Data, DeriveInput, Expr, ExprPath, PatIdent, Path, PathArguments, Type,
    TypeGroup, TypePath, parse::Parse, parse_macro_input,
};
use syn::{ExprGroup, Ident, LitInt, Token};

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

#[proc_macro]
pub fn dyn_serde_trait(input: TokenStream) -> TokenStream {
    let input: ParsedInput = parse_macro_input!(input);
    let trait_name = input.trait_name;
    let seed_name = format_ident!("{}Seed", trait_name);

    quote! {
        impl Serialize for dyn #trait_name {
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

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(deserialize_seed_xxx), forward_attrs(allow, doc, cfg))]
struct DeserializeSeedXXXAttributes {
    seed: TypePath,
}

#[derive(FromField, Debug)]
#[darling(attributes(deserialize_seed_xxx), forward_attrs(allow, doc, cfg))]
struct DeserializeSeedXXXFieldAttributes {
    seed: Option<Expr>,
}

#[derive(FromField, Debug)]
#[darling(attributes(serde), forward_attrs(allow, doc, cfg))]
struct SerdeFieldAttributes {
    #[darling(default)]
    skip: bool,
    with: Option<String>,
}

#[proc_macro_derive(DeserializeSeedXXX, attributes(deserialize_seed_xxx))]
pub fn deserialize_field(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let ident = &input.ident;
    let name = input.ident.to_string();
    let attrs = DeserializeSeedXXXAttributes::from_derive_input(&input).expect("Wrong attributes");

    let attr_seed: TypePath = attrs.seed;
    let attr_seed_args = match attr_seed.path.segments.last().unwrap().arguments.clone() {
        PathArguments::None => vec![],
        PathArguments::AngleBracketed(arguments) => arguments.args.into_iter().collect(),
        PathArguments::Parenthesized(_) => todo!(),
    };

    let fields = match &input.data {
        Data::Struct(v) => &v.fields,
        _ => panic!("This macro only supports structs"),
    };

    struct FieldRecipe {
        field_ident: Ident,
        variant_ident: Ident,
        locale_variable_ident: Ident,
        field_name: String,
        var_decl: proc_macro2::TokenStream,
        key_arm: proc_macro2::TokenStream,
        value_arm: proc_macro2::TokenStream,
        check_missing: proc_macro2::TokenStream,
        skip: bool,
    }

    let fields: Vec<FieldRecipe> = fields.into_iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("Must have ident").clone();
        let field_type: Type = field.ty.clone();
        let field_name = field_ident.to_string().to_case(Case::Camel);
        let variant_ident = Ident::new(&field_ident.to_string().to_case(Case::Pascal), field_ident.span());
        let locale_variable_ident = Ident::new(&field_ident.to_string().to_case(Case::Camel), field_ident.span());
        let key_arm = quote! { #field_name => Ok(Field::#variant_ident) };
        let var_decl = quote! { let mut #locale_variable_ident: Option<#field_type> = None; };
        let serde_options = SerdeFieldAttributes::from_field(field).expect("Wrong serde attributes");
        let skip = serde_options.skip;



        let value_arm = match DeserializeSeedXXXFieldAttributes::from_field(field).expect("Wrong attributes").seed {



            Some(seed) => quote! {
                Field::#variant_ident => {
                    if #locale_variable_ident.is_some() {
                        return Err(serde::de::Error::duplicate_field(#field_name));
                    }
                    #locale_variable_ident = Some(map.next_value_seed(#seed.clone())?.into());
                }
            },
            None => match serde_options.with {
                Some(with) => {
                    let with = Path::from_string(&with).unwrap();
                    quote! {
                    Field::#variant_ident => {
                        struct Seed;

                        impl<'de> serde::de::DeserializeSeed<'de> for Seed {
                            type Value = #field_type;

                            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                            where
                                D: serde::de::Deserializer<'de>,
                            {
                                #with::deserialize(deserializer)
                            }
                        }


                        if #locale_variable_ident.is_some() {
                            return Err(serde::de::Error::duplicate_field(#field_name));
                        }
                        #locale_variable_ident = Some(map.next_value_seed(Seed)?);
                        // #locale_variable_ident = Some(map.next_value_seed(std::marker::PhantomData)?);
                    }
                }
                },
                None => quote! {
                    Field::#variant_ident => {
                        if #locale_variable_ident.is_some() {
                            return Err(serde::de::Error::duplicate_field(#field_name));
                        }
                        #locale_variable_ident = Some(map.next_value()?);
                    }
                }
            }
        };

        let check_missing = quote! {
            let #locale_variable_ident: #field_type = #locale_variable_ident.ok_or_else(|| serde::de::Error::missing_field(#field_name))?;
        };

        FieldRecipe { field_ident, variant_ident, locale_variable_ident, field_name, var_decl, key_arm, value_arm, check_missing, skip, }
    }).collect();

    let expected_root: String = format!("struct {}", ident);
    let expected_key: String = fields
        .iter()
        .filter_map(|f| if f.skip { None } else {Some(format!("`{}`", f.field_name))})
        .intersperse(", ".to_string())
        .collect();

    let field_assignments: Vec<_> = fields.iter().map(|f|{
        let field_ident = &f.field_ident;
        if f.skip {
            quote! { #field_ident: Default::default() }
        } else {
            quote! { #field_ident }
        }}).collect();

    let variant_idents: Vec<_> = fields.iter().filter_map(|f| if f.skip { None } else { Some( f.variant_ident.clone())}).collect();
    let locale_variable_idents: Vec<_> = fields
        .iter()
        .filter_map(|f| if f.skip { None } else { Some(f.locale_variable_ident.clone())})
        .collect();
    let field_names: Vec<_> = fields.iter().filter_map(|f| if f.skip { None } else { Some(  f.field_name.clone())}).collect();
    let var_decls: Vec<_> = fields.iter().filter_map(|f| if f.skip { None } else { Some( f.var_decl.clone())}).collect();
    let key_arms: Vec<_> = fields.iter().filter_map(|f| if f.skip { None } else { Some( f.key_arm.clone())}).collect();
    let value_arms: Vec<_> = fields.iter().filter_map(|f| if f.skip { None } else { Some( f.value_arm.clone())}).collect();
    let check_missings: Vec<_> = fields.iter().filter_map(|f| if f.skip { None } else { Some( f.check_missing.clone())}).collect();

    let output = quote! {
        impl<'de, #(#attr_seed_args),*> serde::de::DeserializeSeed<'de> for #attr_seed {
            type Value = #ident;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                pub const NAMES: &[&str] = &[ #(#field_names),* ];

                enum Field { #(#variant_idents),* }

                impl<'de> serde::de::Deserialize<'de> for Field {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::de::Deserializer<'de>,
                    {
                        struct FieldVisitor;

                        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str(#expected_key)
                            }

                            fn visit_str<E>(self, value: &str) -> Result<Field, E>
                            where
                                E: serde::de::Error,
                            {
                                match value {
                                    #(#key_arms),*,
                                    _ => Err(serde::de::Error::unknown_field(value, NAMES)),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(FieldVisitor)
                    }
                }

                pub struct Visitor <#(#attr_seed_args),*> {
                    seed: #attr_seed
                }

                impl<'ctx, 'de> serde::de::Visitor<'de> for Visitor<'ctx> {
                    type Value = #ident;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(#expected_root)
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                    where
                        V: serde::de::MapAccess<'de>,
                    {
                        #(#var_decls)*
                        while let Some(key) = map.next_key()? {
                            match key {
                                #(#value_arms),*,
                            }
                        }
                        #(#check_missings)*
                        Ok(#ident {
                            #(#field_assignments),*,
                        })
                    }
                }

                deserializer.deserialize_struct(
                    #name,
                    NAMES,
                    Visitor { seed: self },
                )
            }
        }
    };

    println!("macro output: {}", output);

    output.into()
}
