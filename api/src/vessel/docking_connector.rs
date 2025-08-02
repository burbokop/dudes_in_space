use crate::utils::non_nil_uuid::NonNilUuid;
use serde::{Deserialize, Serialize};

pub type DockingConnectorId = NonNilUuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DockingConnector {
    id: DockingConnectorId,
    compat_type: usize,
}

impl DockingConnector {
    pub fn new(compat_type: usize) -> Self {
        Self {
            id: DockingConnectorId::new_v4(),
            compat_type,
        }
    }

    pub fn id(&self) -> DockingConnectorId {
        self.id
    }

    pub fn compat_type(&self) -> usize {
        todo!()
    }
}
