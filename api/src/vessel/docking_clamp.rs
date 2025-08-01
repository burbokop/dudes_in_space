use crate::module::Module;
use crate::utils::tagged_option::TaggedOptionSeed;
use crate::vessel::{Vessel, VesselSeed};
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;

#[derive(Debug, Default, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::DockingClampSeed::<'v>)]
pub struct DockingClamp {
    #[serde(with = "crate::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.vessel_seed)]
    vessel: Option<Vessel>,
}

#[derive(Clone)]
pub struct DockingClampSeed<'v> {
    vessel_seed: TaggedOptionSeed<VesselSeed<'v>>,
}

impl<'v> DockingClampSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> DockingClampSeed<'v> {
        Self {
            vessel_seed: TaggedOptionSeed::new(VesselSeed::new(vault)),
        }
    }
}

impl DockingClamp {
    #[deprecated = "Use is_empty instead"]
    pub fn is_docked(&self) -> bool {
        self.vessel.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.vessel.is_none()
    }

    pub fn dock(&mut self, vessel: Vessel) -> bool {
        if self.vessel.is_some() {
            false
        } else {
            self.vessel = Some(vessel);
            true
        }
    }

    pub fn undock(&mut self) -> Option<Vessel> {
        todo!()
    }

    pub fn vessel_docked(&self) -> Option<&Vessel> {
        self.vessel.as_ref()
    }

    pub fn vessel_docked_mut(&mut self) -> Option<&mut Vessel> {
        todo!()
    }
}
