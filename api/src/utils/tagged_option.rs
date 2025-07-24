use serde::de::value::MapAccessDeserializer;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tagged_option_state")]
enum TaggedOption<T> {
    Some(T),
    None,
}

impl<T> From<Option<T>> for TaggedOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => TaggedOption::Some(value),
            None => TaggedOption::None,
        }
    }
}

impl<T> From<TaggedOption<T>> for Option<T> {
    fn from(value: TaggedOption<T>) -> Self {
        match value {
            TaggedOption::Some(value) => Some(value),
            TaggedOption::None => None,
        }
    }
}

pub fn serialize<T, S>(value: &Option<T>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let t: TaggedOption<_> = value.as_ref().into();
    t.serialize(s)
}

pub fn deserialize<'de, T: Deserialize<'de>, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
{
    let t: TaggedOption<T> = Deserialize::deserialize(deserializer)?;
    Ok(t.into())
}

#[derive(Clone)]
pub struct TaggedOptionSeed<T> {
    element_seed: T,
}

impl<T> TaggedOptionSeed<T> {
    pub fn new(element_seed: T) -> Self {
        Self { element_seed }
    }
}

impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for TaggedOptionSeed<T> {
    type Value = Option<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaggedOptionVisitor<T> {
            element_seed: T,
        }

        impl<'de, T: DeserializeSeed<'de>> Visitor<'de> for TaggedOptionVisitor<T> {
            type Value = Option<T::Value>;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("map")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let key = map.next_key::<String>()?;
                if key.as_deref() != Some("tagged_option_state") {
                    return Err(serde::de::Error::missing_field("tagged_option_state"));
                }

                #[derive(Deserialize)]
                enum TaggedOptionState {
                    Some,
                    None,
                }

                match map.next_value::<TaggedOptionState>()? {
                    TaggedOptionState::Some => Ok(Some(
                        self.element_seed
                            .deserialize(MapAccessDeserializer::new(map))?,
                    )),
                    TaggedOptionState::None => Ok(None),
                }
            }
        }

        deserializer.deserialize_map(TaggedOptionVisitor {
            element_seed: self.element_seed,
        })
    }
}
