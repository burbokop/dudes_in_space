use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
enum UntaggedOption<T> {
    Some(T),
    None,
}

impl<T> From<Option<T>> for UntaggedOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => UntaggedOption::Some(value),
            None => UntaggedOption::None,
        }
    }
}

impl<T> From<UntaggedOption<T>> for Option<T> {
    fn from(value: UntaggedOption<T>) -> Self {
        match value {
            UntaggedOption::Some(value) => Some(value),
            UntaggedOption::None => None,
        }
    }
}

pub fn serialize<T, S>(value: &Option<T>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let t: UntaggedOption<_> = value.as_ref().into();
    t.serialize(s)
}

pub fn deserialize<'de, T: Deserialize<'de>, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
{
    let t: UntaggedOption<T> = Deserialize::deserialize(deserializer)?;
    Ok(t.into())
}
