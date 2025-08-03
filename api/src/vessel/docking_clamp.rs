use crate::module::{Module, ModuleCapability, ProcessTokenContext};
use crate::utils::tagged_option::TaggedOptionSeed;
use crate::vessel::docking_connector::DockingConnectorId;
use crate::vessel::{Vessel, VesselSeed};
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::person::{Logger, ObjectiveDeciderVault};

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::docking_clamp::DockingClampConnectionSeed::<'v>)]
pub struct DockingClampConnection {
    #[deserialize_seed_xxx(seed = self.seed.vessel_seed)]
    pub vessel: Vessel,
    pub connector_id: DockingConnectorId,
}

#[derive(Clone)]
struct DockingClampConnectionSeed<'v> {
    vessel_seed: VesselSeed<'v>,
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::DockingClampSeed::<'v>)]
pub struct DockingClamp {
    #[serde(with = "crate::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.connection_seed)]
    connection: Option<DockingClampConnection>,
    compat_type: usize,
}

#[derive(Clone)]
pub struct DockingClampSeed<'v> {
    connection_seed: TaggedOptionSeed<DockingClampConnectionSeed<'v>>,
}

impl<'v> DockingClampSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> DockingClampSeed<'v> {
        Self {
            connection_seed: TaggedOptionSeed::new(DockingClampConnectionSeed {
                vessel_seed: VesselSeed::new(vault),
            }),
        }
    }
}

impl DockingClamp {
    pub fn new(compat_type: usize) -> DockingClamp {
        DockingClamp {
            connection: None,
            compat_type,
        }
    }

    #[deprecated = "Use is_empty instead"]
    pub fn is_docked(&self) -> bool {
        self.connection.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.connection.is_none()
    }

    pub fn dock_by_connector_id(
        &mut self,
        vessel: Vessel,
        connector_id: DockingConnectorId,
    ) -> Result<(), DockByConnectorId> {
        if self.connection.is_some() {
            Err(DockByConnectorId::Busy)
        } else {
            self.connection = Some(DockingClampConnection {
                vessel,
                connector_id,
            });
            Ok(())
        }
    }

    pub fn dock(&mut self, vessel: Vessel) -> Result<(), DockError> {
        if self.connection.is_some() {
            Err(DockError::Busy)
        } else {
            let connector_id = vessel
                .modules_with_capability(ModuleCapability::DockingConnector)
                .map(|x| {
                    x.docking_connectors()
                        .into_iter()
                        .map(|x| x.id())
                        .collect::<Vec<_>>()
                })
                .flatten()
                .next()
                .ok_or(DockError::ConnectorNotFound)?;

            self.connection = Some(DockingClampConnection {
                vessel,
                connector_id,
            });
            Ok(())
        }
    }

    pub fn undock(&mut self) -> Option<Vessel> {
        todo!()
    }

    pub fn connection(&self) -> Option<&DockingClampConnection> {
        self.connection.as_ref()
    }

    pub fn connection_mut(&mut self) -> Option<&mut DockingClampConnection> {
        todo!()
    }

    pub fn proceed(
        &mut self,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        if let Some(connection) = self.connection.as_mut() {
            connection.vessel.proceed(process_token_context, decider_vault, logger);
        }
    }
}

#[derive(Debug)]
pub enum DockError {
    Busy,
    ConnectorNotFound,
}

impl Display for DockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for DockError {}

#[derive(Debug)]
pub enum DockByConnectorId {
    Busy,
}

impl Display for DockByConnectorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for DockByConnectorId {}
