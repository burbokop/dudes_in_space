use std::cell::RefCell;
use serde::de::{DeserializeSeed, Error as _, SeqAccess, Visitor};
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_intermediate::Intermediate;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Formatter;
use std::rc::Rc;
use crate::bl::modules::Module;

pub(crate) type TypeId = String;

pub(crate) trait DynSerialize {
    fn type_id(&self) -> TypeId;
    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>>;
}

pub(crate) trait DynDeserializeSeed<T: ?Sized> {
    fn type_id(&self) -> TypeId;
    fn deserialize(&self, intermediate: Intermediate) -> Result<Box<T>, Box<dyn Error>>;
}

pub(crate) fn dyn_serialize<S: Serializer, T: ?Sized + DynSerialize>(
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

pub(crate) struct DynDeserializeSeedVault<T: ?Sized> {
    data: BTreeMap<String, Box<dyn DynDeserializeSeed<T>>>,
}

impl<T: ?Sized> DynDeserializeSeedVault<T> {
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub(crate) fn with<F: DynDeserializeSeed<T> + 'static>(mut self, sd: F) -> Self {
        let name = sd.type_id();
        self.data.insert(name, Box::new(sd));
        self
    }

    pub(crate) fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
    ) -> Result<Box<T>, D::Error> {
        #[derive(Deserialize)]
        struct Impl {
            tp: String,
            payload: Intermediate,
        }

        let i = Impl::deserialize(deserializer)?;

        let deser = self
            .data
            .get(&i.tp)
            .expect(&format!("Module with id `{}` not found", i.tp));

        Ok(deser
            .deserialize(i.payload)
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
pub(crate) struct VecSeed<T> {
    element_seed: T,
}

impl<T> VecSeed<T> {
    pub(crate) fn new(element_seed: T) -> Self { Self { element_seed } }
}

impl<'de, T: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for VecSeed<T> {
    type Value = Vec<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>
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

        deserializer.deserialize_seq(VecVisitor { element_seed: self.element_seed })
    }
}