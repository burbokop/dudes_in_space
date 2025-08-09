#![deny(warnings)]

use serde::de::{DeserializeSeed, Error as _, SeqAccess, Visitor};
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_intermediate::Intermediate;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Formatter;
use std::rc::Rc;

pub type TypeId = String;

pub trait DynSerialize {
    fn type_id(&self) -> TypeId;
    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>>;
}

pub trait DynDeserializeSeed<T: ?Sized> {
    fn type_id(&self) -> TypeId;
    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<T>,
    ) -> Result<Box<T>, Box<dyn Error>>;
}

pub fn dyn_serialize<S: Serializer, T: ?Sized + DynSerialize>(
    serializer: S,
    module: &T,
) -> Result<S::Ok, S::Error> {
    let type_id = module.type_id();

    #[derive(Serialize)]
    struct Impl {
        tp: TypeId,
        payload: Intermediate,
    }

    Impl {
        tp: type_id,
        payload: module.serialize().map_err(|e| S::Error::custom(e))?,
    }
    .serialize(serializer)
}

pub struct DynDeserializeSeedVault<T: ?Sized> {
    data: BTreeMap<String, Box<dyn DynDeserializeSeed<T>>>,
}

impl<T: ?Sized> Default for DynDeserializeSeedVault<T> {
    fn default() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
}

impl<T: ?Sized> DynDeserializeSeedVault<T> {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn with<F: DynDeserializeSeed<T> + 'static>(mut self, sd: F) -> Self {
        let name = sd.type_id();
        self.data.insert(name, Box::new(sd));
        self
    }

    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
    ) -> Result<Box<T>, D::Error> {
        #[derive(Deserialize)]
        struct Impl {
            tp: String,
            payload: Intermediate,
        }

        let i = Impl::deserialize(deserializer)?;

        let deser = self.data.get(&i.tp).ok_or_else(|| {
            D::Error::custom(format!(
                "{} with id `{}` not found",
                std::any::type_name::<T>(),
                i.tp
            ))
        })?;

        Ok(deser
            .deserialize(i.payload, self)
            .map_err(|e| D::Error::custom(e))?)
    }
}

pub fn from_intermediate_seed<'a, T>(
    seed: T,
    value: &'a Intermediate,
) -> serde_intermediate::error::Result<T::Value>
where
    T: DeserializeSeed<'a>,
{
    seed.deserialize(
        serde_intermediate::de::intermediate::Deserializer::from_intermediate(
            value,
            Default::default(),
        ),
    )
}

#[derive(Clone)]
pub struct VecSeed<T> {
    element_seed: T,
}

impl<T> VecSeed<T> {
    pub fn new(element_seed: T) -> Self {
        Self { element_seed }
    }
}

impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for VecSeed<T> {
    type Value = Vec<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VecVisitor<T> {
            element_seed: T,
        }

        impl<'de, T: DeserializeSeed<'de> + Clone> Visitor<'de> for VecVisitor<T> {
            type Value = Vec<T::Value>;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("list")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut elements: Vec<T::Value> = Default::default();
                while let Some(key) = seq.next_element_seed(self.element_seed.clone())? {
                    elements.push(key);
                }
                Ok(elements)
            }
        }

        deserializer.deserialize_seq(VecVisitor {
            element_seed: self.element_seed,
        })
    }
}

#[derive(Clone)]
pub struct OptionSeed<T> {
    element_seed: T,
}

impl<T> OptionSeed<T> {
    pub fn new(element_seed: T) -> Self {
        Self { element_seed }
    }
}

impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for OptionSeed<T> {
    type Value = Option<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptionVisitor<T> {
            element_seed: T,
        }

        impl<'de, T: DeserializeSeed<'de>> Visitor<'de> for OptionVisitor<T> {
            type Value = Option<T::Value>;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("option")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Some(self.element_seed.deserialize(deserializer)?))
            }
        }

        deserializer.deserialize_option(OptionVisitor {
            element_seed: self.element_seed,
        })
    }
}

#[derive(Clone)]
pub struct BoxSeed<T> {
    element_seed: T,
}

impl<T> BoxSeed<T> {
    pub fn new(element_seed: T) -> Self {
        Self { element_seed }
    }
}

impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for BoxSeed<T> {
    type Value = Box<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Box::new(self.element_seed.deserialize(deserializer)?))
    }
}
