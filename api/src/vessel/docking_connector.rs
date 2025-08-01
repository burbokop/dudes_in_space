use crate::utils::non_nil_uuid::NonNilUuid;

pub type DockingConnectorId = NonNilUuid;

pub struct DockingConnector {
    id: DockingConnectorId,
    compat_type: usize,
}

impl DockingConnector {
    pub fn id(&self) -> DockingConnectorId{
        todo!()
    }
    
    pub fn compat_type(&self) -> usize{
todo!()        
    }
}