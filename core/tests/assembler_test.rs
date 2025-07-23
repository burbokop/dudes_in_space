use dudes_in_space_api::Person;
use dudes_in_space_api::modules::{Module, ModuleFactory};
use dudes_in_space_core::modules::{Assembler, AssemblerSeed};
use dyn_serde::{DynDeserializeSeedVault, from_intermediate_seed};
use rand::rng;
use serde_intermediate::TextConfigStyle::Default;
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::collections::HashMap;

fn map_to_struct(v: Intermediate) -> Intermediate {
    Intermediate::Struct(
        v.as_map()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.as_str().unwrap().to_string(), v.clone()))
            .collect(),
    )
}

#[test]
fn serde() {
    let mut assembler = Assembler::new(vec![]);
    assert!(assembler.can_insert_person());
    assert!(assembler.insert_person(Person::random(&mut rng())));
    assert!(!assembler.can_insert_person());

    let intermediate = to_intermediate(&assembler).unwrap();

    // let intermediate = intermediate.as_map().unwrap();
    // let intermediate = intermediate.as_new_type_struct().unwrap().clone();
    // let intermediate = Intermediate::Map(intermediate.as_map().unwrap().to_vec());

    let json = serde_json::to_string(&intermediate).unwrap();

    println!("json: {}", json);

    let parsed_intermediate: Intermediate = serde_json::from_str(&json).unwrap();

    // println!("Before: {:?}", parsed_intermediate);

    // let parsed_intermediate = Intermediate::Map(parsed_intermediate.as_map().unwrap().to_vec());

    let parsed_intermediate = map_to_struct(parsed_intermediate);

    // println!("After : {:?}", parsed_intermediate);

    // assert_eq!(parsed_intermediate, intermediate);

    let vault = DynDeserializeSeedVault::<dyn ModuleFactory>::new();

    let parsed_assembler: Assembler =
        from_intermediate_seed(AssemblerSeed::new(&vault), &parsed_intermediate).unwrap();
}

mod xxxxxxx {

    use std::fmt;

    use dudes_in_space_api::Person;
    use serde::de::{
        self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor,
    };

    #[allow(dead_code)]
    struct Duration {
        secs: u64,
        nanos: u32,
    }

    impl Duration {
        fn new(_: u64, _: u32) -> Self {
            unimplemented!()
        }
    }

    impl<'de> Deserialize<'de> for Duration {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            enum Field {
                Secs,
                Nanos,
            };

            // This part could also be generated independently by:
            //
            //    #[derive(Deserialize)]
            //    #[serde(field_identifier, rename_all = "lowercase")]
            //    enum Field { Secs, Nanos }
            impl<'de> Deserialize<'de> for Field {
                fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> Visitor<'de> for FieldVisitor {
                        type Value = Field;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("`secs` or `nanos`")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where
                            E: de::Error,
                        {
                            match value {
                                "secs" => Ok(Field::Secs),
                                "nanos" => Ok(Field::Nanos),
                                _ => Err(de::Error::unknown_field(value, FIELDS)),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }

            struct DurationVisitor;

            impl<'de> Visitor<'de> for DurationVisitor {
                type Value = Duration;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct Duration")
                }

                fn visit_seq<V>(self, mut seq: V) -> Result<Duration, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    let secs = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                    let nanos = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                    Ok(Duration::new(secs, nanos))
                }

                fn visit_map<V>(self, mut map: V) -> Result<Duration, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut secs = None;
                    let mut nanos = None;
                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Secs => {
                                if secs.is_some() {
                                    return Err(de::Error::duplicate_field("secs"));
                                }
                                secs = Some(map.next_value()?);
                            }
                            Field::Nanos => {
                                if nanos.is_some() {
                                    return Err(de::Error::duplicate_field("nanos"));
                                }
                                nanos = Some(map.next_value()?);
                            }
                        }
                    }
                    let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
                    let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
                    Ok(Duration::new(secs, nanos))
                }
            }

            const FIELDS: &'static [&'static str] = &["secs", "nanos"];
            deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
        }
    }

    struct Seed;

    impl<'de> DeserializeSeed<'de> for Seed {
        type Value = Option<Person>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            todo!()
        }
    }
}
