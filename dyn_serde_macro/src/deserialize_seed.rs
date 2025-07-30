use convert_case::{Case, Casing};
use darling::{FromField, FromMeta};
use proc_macro::TokenStream;
use quote::{ format_ident, quote};
use syn::spanned::Spanned;
use syn::{
    self, Data, DeriveInput, Expr,  Fields, GenericArgument,  Path,
    PathArguments, Type,  TypePath, Variant,  parse_macro_input,
};
use syn::{ Ident, };

#[derive(deluxe::ExtractAttributes, Debug)]
#[deluxe(attributes(deserialize_seed_xxx))]
struct DeserializeSeedXXXAttributes {
    seed: TypePath,
}

#[derive(FromField, Debug)]
#[darling(attributes(deserialize_seed_xxx), forward_attrs(allow, doc, cfg))]
struct DeserializeSeedXXXFieldAttributes {
    seed: Option<Expr>,
}

#[derive(deluxe::ExtractAttributes, Debug)]
#[deluxe(attributes(deserialize_seed_xxx))]
struct DeserializeSeedXXXVariantAttributes {
    seeds: Vec<(Ident, Expr)>,
}

#[derive(deluxe::ExtractAttributes, Debug)]
#[deluxe(attributes(serde))]
struct SerdeAttributes {
    tag: Option<String>,
}

#[derive(FromField, Debug)]
#[darling(attributes(serde), forward_attrs(allow, doc, cfg))]
struct SerdeFieldAttributes {
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    skip_deserializing: bool,
    with: Option<String>,
}

pub(crate) fn deserialize_seed_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let ident = &input.ident;
    let name = input.ident.to_string();

    let attrs: DeserializeSeedXXXAttributes =
        deluxe::extract_attributes(&mut input.attrs.clone()).expect("Wrong attributes");
    let serde_attrs: SerdeAttributes =
        deluxe::extract_attributes(&mut input.attrs.clone()).expect("Wrong attributes");

    let mut seed_type: TypePath = attrs.seed;
    let seed_generic_args = match seed_type.path.segments.last().unwrap().arguments.clone() {
        PathArguments::None => vec![],
        PathArguments::AngleBracketed(arguments) => arguments.args.into_iter().collect(),
        PathArguments::Parenthesized(_) => todo!(),
    };
    seed_type.path.segments.last_mut().unwrap().arguments = PathArguments::None;
    let seed_type = seed_type;

    let output = match &input.data {
        Data::Struct(data) => {
            assert!(serde_attrs.tag.is_none());
            deserialize_seed_struct(
                name,
                ident,
                seed_generic_args,
                seed_type,
                &data.fields,
                vec![],
            )
            .into()
        }
        Data::Enum(data) => match serde_attrs.tag {
            None => deserialize_seed_enum(
                name,
                ident,
                seed_generic_args,
                seed_type,
                data.variants.iter().collect(),
            ),
            Some(tag) => deserialize_seed_tagged_enum(
                name,
                ident,
                seed_generic_args,
                seed_type,
                data.variants.iter().collect(),
                tag,
            ),
        },
        Data::Union(_) => panic!("This macro don't support unions"),
    };
    println!(
        "`dyn_serde_macro::DeserializeSeedXXX` macro output: {}",
        output
    );
    output
}

fn deserialize_seed_struct(
    name: String,
    ident: &Ident,
    seed_generic_args: Vec<GenericArgument>,
    seed_type: TypePath,
    fields: &Fields,
    extra_field_seeds: Vec<(Ident, Expr)>,
) -> proc_macro2::TokenStream {
    let visitor = deserialize_seed_struct_visitor(
        ident,
        seed_generic_args.clone(),
        seed_type.clone(),
        fields,
        extra_field_seeds,
    );

    quote! {
        impl<'de, #(#seed_generic_args),*> serde::de::DeserializeSeed<'de> for #seed_type <#(#seed_generic_args),*> {
            type Value = #ident;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                #visitor

                deserializer.deserialize_struct(
                    #name,
                    NAMES,
                    Visitor { seed: self },
                )
            }
        }
    }
    .into()
}

fn deserialize_seed_struct_visitor(
    ident: &Ident,
    seed_generic_args: Vec<GenericArgument>,
    seed_type: TypePath,
    fields: &Fields,
    extra_field_seeds: Vec<(Ident, Expr)>,
) -> proc_macro2::TokenStream {
    struct FieldRecipe {
        field_ident: Ident,
        variant_ident: Ident,
        field_name: String,
        var_decl: proc_macro2::TokenStream,
        key_arm: proc_macro2::TokenStream,
        value_arm: proc_macro2::TokenStream,
        check_missing: proc_macro2::TokenStream,
        skip: bool,
    }

    let fields: Vec<FieldRecipe> = fields.into_iter().enumerate().map(|(index, field)| {
        let field_ident = field.ident.as_ref().unwrap_or(&format_ident!("field{}", index)).clone();
        let field_type: Type = field.ty.clone();
        let field_name = field_ident.to_string();
        let variant_ident = Ident::new(&field_ident.to_string().to_case(Case::Pascal), field_ident.span());
        let locale_variable_ident = Ident::new(&field_ident.to_string().to_case(Case::Snake), field_ident.span());
        let key_arm = quote! { #field_name => Ok(Field::#variant_ident) };
        let var_decl = quote! { let mut #locale_variable_ident: Option<#field_type> = None; };
        let serde_options = SerdeFieldAttributes::from_field(field).expect("Wrong serde attributes");
        let skip = serde_options.skip || serde_options.skip_deserializing;

        let seed: Option<Expr> = DeserializeSeedXXXFieldAttributes::from_field(field).expect("Wrong attributes").seed;
        let extra_seed = extra_field_seeds.iter().find_map(|(ident, seed)| if ident == &field_ident { Some(seed) } else { None });

        println!("xxx_field: {} -> {:?} | {:?}", field_ident, seed, extra_seed);

        assert!(!(seed.is_some() && extra_seed.is_some()));

        let value_arm = match seed.or(extra_seed.cloned()) {
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

                        impl<'__de> serde::de::DeserializeSeed<'__de> for Seed {
                            type Value = #field_type;

                            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                            where
                                D: serde::de::Deserializer<'__de>,
                            {
                                #with::deserialize(deserializer)
                            }
                        }

                        if #locale_variable_ident.is_some() {
                            return Err(serde::de::Error::duplicate_field(#field_name));
                        }
                        #locale_variable_ident = Some(map.next_value_seed(Seed)?);
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

        FieldRecipe { field_ident, variant_ident, field_name, var_decl, key_arm, value_arm, check_missing, skip, }
    }).collect();

    let expected_root: String = format!("struct {}", ident);
    let expected_key: String = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(format!("`{}`", f.field_name))
            }
        })
        .intersperse(", ".to_string())
        .collect();

    let field_assignments: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            if f.skip {
                quote! { #field_ident: Default::default() }
            } else {
                quote! { #field_ident }
            }
        })
        .collect();

    let variant_idents: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(f.variant_ident.clone())
            }
        })
        .collect();

    let field_names: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(f.field_name.clone())
            }
        })
        .collect();

    let var_decls: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(f.var_decl.clone())
            }
        })
        .collect();

    let key_arms: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(f.key_arm.clone())
            }
        })
        .collect();

    let value_arms: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(f.value_arm.clone())
            }
        })
        .collect();

    let check_missings: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if f.skip {
                None
            } else {
                Some(f.check_missing.clone())
            }
        })
        .collect();

    quote! {
        const NAMES: &[&str] = &[ #(#field_names),* ];

        enum Field { #(#variant_idents),* }
        impl<'__de> serde::de::Deserialize<'__de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: serde::de::Deserializer<'__de>,
            {
                struct FieldVisitor;

                impl<'__de> serde::de::Visitor<'__de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(#expected_key)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error,
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

        struct Visitor <#(#seed_generic_args),*> {
            seed: #seed_type <#(#seed_generic_args),*>
        }

        impl<#(#seed_generic_args),*, '__de> serde::de::Visitor<'__de> for Visitor<#(#seed_generic_args),*> {
            type Value = #ident;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(#expected_root)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'__de>,
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
    }
        .into()
}

fn deserialize_seed_enum(
    name: String,
    ident: &Ident,
    seed_generic_args: Vec<GenericArgument>,
    seed_type: TypePath,
    variants: Vec<&Variant>,
) -> TokenStream {
    println!("deserialize_seed_enum: {:#?}", variants);

    struct VariantRecipe {
        name: String,
        ident: Ident,
        struct_declaration: proc_macro2::TokenStream,
        arm: proc_macro2::TokenStream,
    }

    let variants: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = variant.ident.clone();
            let struct_ident = format_ident!("__{}_{}", ident, variant.ident);
            let fields = variant.fields.clone();

            let variant_attr: Option<DeserializeSeedXXXVariantAttributes> = match variant.fields {
                Fields::Named(_) | Fields::Unnamed(_) => Some(
                    deluxe::extract_attributes(&mut variant.attrs.clone())
                        .expect("Wrong attributes"),
                ),
                Fields::Unit => None,
            };

            let struct_visitor = variant_attr.map(|variant_attr| {
                println!("variant_attr: {:#?}", variant_attr);

                deserialize_seed_struct_visitor(
                    &struct_ident,
                    seed_generic_args.clone(),
                    seed_type.clone(),
                    &fields,
                    variant_attr.seeds,
                )
            });

            let fields_copy_list = fields
                .iter()
                .enumerate()
                .map(|(index,f)|f.ident.clone().unwrap_or(Ident::new(&format!("{}",index) , f.span())))
                .map(|f|quote!{ #f: self.#f });

            VariantRecipe {
                name: variant.ident.to_string(),
                ident: variant.ident.clone(),
                struct_declaration: if struct_visitor.is_some() {
                    quote! {
                        struct #struct_ident #fields;

                        impl #struct_ident {
                            fn into_dst(self) -> #ident {
                                #ident :: #variant_ident { #(#fields_copy_list),*, }
                            }
                        }
                    }
                } else {
                    quote! {
                        #[derive(Deserialize)]
                        struct #struct_ident #fields;
                    }
                }
                    .into(),
                arm: match struct_visitor {
                    None => quote! {
                        (Discriminant::#variant_ident, variant) => Ok(Self::Value::#variant_ident)
                    },
                    Some(_) => quote! {
                        (Discriminant::#variant_ident, variant) => {
                            use serde::de::VariantAccess as _;

                            #struct_visitor

                            let value: #struct_ident = variant.struct_variant(NAMES, Visitor { seed: self.seed })?;

                            Ok(value.into_dst())
                        }
                    },
                }
                    .into(),
            }
        })
        .collect();

    let variant_names = variants.iter().map(|v| v.name.clone());
    let variant_idents = variants.iter().map(|v| v.ident.clone());
    let variant_struct_declarations = variants.iter().map(|v| v.struct_declaration.clone());
    let variant_arms = variants.iter().map(|v| v.arm.clone());

    quote! {
        impl<'__de, #(#seed_generic_args),*> serde::de::DeserializeSeed<'__de> for #seed_type <#(#seed_generic_args),*> {
            type Value = #ident;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'__de>,
            {
                #[derive(Deserialize)]
                enum Discriminant { #(#variant_idents),*, }
                #(#variant_struct_declarations)*

                struct Visitor <#(#seed_generic_args),*> {
                    seed: #seed_type <#(#seed_generic_args),*>
                }

                impl<#(#seed_generic_args),*, '__de> serde::de::Visitor<'__de> for Visitor<#(#seed_generic_args),*> {
                    type Value = #ident;

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "enum {}", #name)
                    }

                    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::EnumAccess<'__de>,
                    {
                        match data.variant()? {
                            #(#variant_arms),*,
                        }
                    }
                }

                deserializer.deserialize_enum(#name, &[#(#variant_names),*,], Visitor { seed: self })
            }
        }
    }
        .into()
}

fn deserialize_seed_tagged_enum(
    name: String,
    ident: &Ident,
    seed_generic_args: Vec<GenericArgument>,
    seed_type: TypePath,
    variants: Vec<&Variant>,
    tag: String,
) -> TokenStream {
    println!("deserialize_seed_tagged_enum: {:#?}", variants);

    struct VariantRecipe {
        name: String,
        ident: Ident,
        struct_declaration: proc_macro2::TokenStream,
        arm: proc_macro2::TokenStream,
    }

    let variants: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = variant.ident.clone();
            let struct_ident = format_ident!("__{}_{}", ident, variant.ident);
            let struct_seed_ident = format_ident!("{}Seed", struct_ident);

            let fields = variant.fields.clone();

            let variant_attr: Option<DeserializeSeedXXXVariantAttributes> = match variant.fields {
                Fields::Named(_) | Fields::Unnamed(_) => Some(
                    deluxe::extract_attributes(&mut variant.attrs.clone())
                        .expect("Wrong attributes"),
                ),
                Fields::Unit => None,
            };

            let struct_seed_impl = variant_attr.map(|variant_attr| {
                println!("variant_attr: {:#?}", variant_attr);



                deserialize_seed_struct(
                    struct_ident.to_string(),
                    &struct_ident,
                    seed_generic_args.clone(),
                    TypePath::from_string(&struct_seed_ident.clone().to_string()).unwrap(),
                    &fields,
                    variant_attr.seeds,
                )
            });

            let fields_copy_list = fields
                .iter()
                .enumerate()
                .map(|(index,f)|f.ident.clone().unwrap_or(Ident::new(&format!("field{}",index) , f.span())))
                .map(|f|quote!{ #f: self.#f });

            VariantRecipe {
                name: variant.ident.to_string(),
                ident: variant.ident.clone(),
                struct_declaration: if struct_seed_impl.is_some() {
                    quote! {
                        struct #struct_ident #fields;

                        impl #struct_ident {
                            fn into_dst(self) -> #ident {
                                #ident :: #variant_ident { #(#fields_copy_list),*, }
                            }
                        }
                    }
                } else {
                    quote! {
                        #[derive(Deserialize)]
                        struct #struct_ident #fields;
                    }
                }
                    .into(),
                arm: match struct_seed_impl {
                    None => quote! {
                        Tag::#variant_ident => Ok(Self::Value::#variant_ident)
                    },
                    Some(_) => quote! {
                        Tag::#variant_ident => {
                            struct #struct_seed_ident <#(#seed_generic_args),*> {
                                seed: #seed_type <#(#seed_generic_args),*>
                            }

                            #struct_seed_impl

                            use serde::de::DeserializeSeed as _;

                            let value: #struct_ident = #struct_seed_ident { seed: self.seed }.deserialize(serde::de::value::MapAccessDeserializer::new(map))?;

                            Ok(value.into_dst())
                        }
                    },
                }
                    .into(),
            }
        })
        .collect();

    let variant_names = variants.iter().map(|v| v.name.clone());
    let variant_idents = variants.iter().map(|v| v.ident.clone());
    let variant_struct_declarations = variants.iter().map(|v| v.struct_declaration.clone());
    let variant_arms = variants.iter().map(|v| v.arm.clone());

    quote! {
        impl<'__de, #(#seed_generic_args),*> serde::de::DeserializeSeed<'__de> for #seed_type <#(#seed_generic_args),*> {
            type Value = #ident;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'__de>,
            {
                #(#variant_struct_declarations)*

                struct Visitor <#(#seed_generic_args),*> {
                    seed: #seed_type <#(#seed_generic_args),*>
                }

                impl<#(#seed_generic_args),*, '__de> serde::de::Visitor<'__de> for Visitor<#(#seed_generic_args),*> {
                    type Value = #ident;

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "map")
                    }

                    fn visit_map<A: serde::de::MapAccess<'__de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                        let key = map.next_key::<String>()?;
                        if key.as_deref() != Some(#tag) {
                            return Err(serde::de::Error::missing_field(#tag));
                        }

                        #[derive(Deserialize)]
                        enum Tag { #(#variant_idents),*, }

                        match map.next_value::<Tag>()? {
                            #(#variant_arms),*,
                        }
                    }
                }

                deserializer.deserialize_enum(#name, &[#(#variant_names),*,], Visitor { seed: self })
            }
        }
    }
        .into()
}
