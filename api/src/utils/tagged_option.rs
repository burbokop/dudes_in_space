use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tagged_option_state")]
enum TaggedOption<T> {
    Some(T),
    None
}

impl<T> From<Option<T>> for TaggedOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => TaggedOption::Some(value),
            None => TaggedOption::None,
        }
    }
}

impl<T> From<TaggedOption<T>> for Option<T>  {
    fn from(value: TaggedOption<T>) -> Self {
        match value {
            TaggedOption::Some(value) => Some(value),
            TaggedOption::None => None,
        }
    }
}

pub fn serialize<T, S>(value: &Option<T>, s: S) -> Result<S::Ok, S::Error>
where S: Serializer, T: Serialize {
    let t: TaggedOption<_> = value.as_ref().into();
    t.serialize(s)
}

pub fn deserialize<'de, T: Deserialize<'de>, D> (deserializer: D) -> Result<Option<T>, D::Error> where D: Deserializer<'de> {
    let t: TaggedOption<T> = Deserialize::deserialize(deserializer)?;
    Ok(t.into())
}
