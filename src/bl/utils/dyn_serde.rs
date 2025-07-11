use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error as _;
use serde::ser::Error as _;
use serde_intermediate::Intermediate;
use crate::bl::modules::{Module, ModuleTypeId};

pub(crate) trait DynSerialize {
    fn type_id(&self) -> String;
    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>>;
}

pub(crate) trait DynDeserializeFactory<T: ?Sized> {
    fn type_id(&self) -> String;
    fn deserialize(&self, intermediate: Intermediate) -> Result<Box<T>, Box<dyn Error>>;
}


pub(crate) struct DynDeserializeFactoryRegistry<T: ?Sized> {
    data: BTreeMap<String, Box<dyn DynDeserializeFactory<T>>>,
}

impl<T: ?Sized> DynDeserializeFactoryRegistry<T> {
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub(crate) fn with<F: DynDeserializeFactory<T> + 'static>(mut self, sd: F) -> Self {
        let name = sd.type_id();
        self.data.insert(name, Box::new(sd));
        self
    }

    pub(crate) fn serialize<S: Serializer>(
        serializer: S,
        module: &RefCell< Box<T>>,
    ) -> Result<S::Ok, S::Error> where T:DynSerialize {
        let module = module.borrow();

        let type_id = module.type_id();

        #[derive(Serialize)]
        struct Impl {
            tp: ModuleTypeId,
            payload: Intermediate,
        }

        Impl{ tp: type_id, payload: module.serialize().map_err(|e|S::Error::custom(e))? }.serialize( serializer)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(&self, deserializer: D) -> Result<Box<T>, D::Error> {
        #[derive(Deserialize)]
        struct Impl {
            tp: String,
            payload: Intermediate,
        }

        let i = Impl::deserialize(deserializer)?;

        let deser = self.data.get(&i.tp).expect(&format!("Module with id `{}` not found", i.tp));

        Ok(deser.deserialize(i.payload).map_err(|e| D::Error::custom(e))?)
    }
}