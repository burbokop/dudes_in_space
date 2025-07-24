use crate::module::Module;
use crate::vessel::{Vessel, VesselId, VesselSeed};
use dyn_serde::{DynDeserializeSeedVault, VecSeed};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::environment::EnvironmentSeed::<'v>)]
pub struct Environment {
    #[deserialize_seed_xxx(seed = self.seed.vessel_seed)]
    vessels: Vec<Vessel>,
}

pub struct EnvironmentSeed<'v> {
    vessel_seed: VecSeed<VesselSeed<'v>>,
}

impl<'v> EnvironmentSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> Self {
        Self {
            vessel_seed: VecSeed::new(VesselSeed::new(vault)),
        }
    }
}

impl Environment {
    pub fn new(vessels: Vec<Vessel>) -> Self {
        Self { vessels }
    }

    pub(crate) fn vessel_by_id(&self, id: VesselId) -> Option<&Vessel> {
        self.vessels.iter().find(|v| v.id() == id)
    }

    pub fn vessel_by_id_mut(&mut self, id: VesselId) -> Option<&mut Vessel> {
        self.vessels.iter_mut().find(|v| v.id() == id)
    }

    pub fn proceed(&mut self) {
        for v in &mut self.vessels {
            v.proceed()
        }
    }
}
