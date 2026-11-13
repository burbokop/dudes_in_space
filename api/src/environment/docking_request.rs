use crate::vessel::VesselId;

/// Requests which change the vessel tree and therefore can be applied only by the environment after all vessels proceeded.
/// Intentionally crate private: they can be issued only by vessels on behalf of their modules (see `VesselModuleInterface`).
#[derive(Debug)]
pub(crate) enum DockingRequest {
    Undock { vessel: VesselId },
    Dock { vessel: VesselId, target: VesselId },
}
