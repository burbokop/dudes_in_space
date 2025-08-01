use crate::module::Module;
use crate::utils::tagged_option::TaggedOptionSeed;
use crate::vessel::{Vessel, VesselSeed};
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use crate::vessel::docking_connector::DockingConnectorId;

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::docking_clamp::DockingClampConnectionSeed::<'v>)]
struct DockingClampConnection {
    #[deserialize_seed_xxx(seed = self.seed.vessel_seed)]
    vessel: Vessel,
    connector_id: DockingConnectorId,
}

#[derive(Clone)]
struct DockingClampConnectionSeed<'v> {
    vessel_seed: VesselSeed<'v>,
}

#[derive(Debug, Default, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::DockingClampSeed::<'v>)]
pub struct DockingClamp {
    #[serde(with = "crate::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.connection_seed)]
    connection: Option<DockingClampConnection>,
}

#[derive(Clone)]
pub struct DockingClampSeed<'v> {
    connection_seed: TaggedOptionSeed<DockingClampConnectionSeed<'v>>,
}

impl<'v> DockingClampSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> DockingClampSeed<'v> {
        Self {
            connection_seed: TaggedOptionSeed::new(DockingClampConnectionSeed { vessel_seed: VesselSeed::new(vault) }),
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


