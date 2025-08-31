use crate::module::{ModuleCapability, ModuleId, ModuleStorage};
use crate::vessel::{DockingClamp, DockingConnectorId, VesselId};
use std::collections::BTreeSet;

pub fn find_docking_clamp_with_vessel_with_id(
    docking_clamps: &[DockingClamp],
    vessel_id: VesselId,
) -> Option<&DockingClamp> {
    docking_clamps.iter().find(|clamp| {
        clamp
            .connection()
            .map(|x| x.vessel.id() == vessel_id)
            .unwrap_or(false)
    })
}

pub fn find_docking_clamp_with_vessel_with_id_mut(
    docking_clamps: &mut [DockingClamp],
    vessel_id: VesselId,
) -> Option<&mut DockingClamp> {
    docking_clamps.iter_mut().find(|clamp| {
        clamp
            .connection()
            .map(|x| x.vessel.id() == vessel_id)
            .unwrap_or(false)
    })
}

pub fn find_docking_clamp_with_connector_with_id(
    docking_clamps: &[DockingClamp],
    connector_id: DockingConnectorId,
) -> Option<&DockingClamp> {
    docking_clamps.iter().find(|clamp| {
        clamp
            .connection()
            .map(|x| x.connector_id == connector_id)
            .unwrap_or(false)
    })
}

pub fn find_docking_clamp_with_connector_with_id_mut(
    docking_clamps: &mut [DockingClamp],
    connector_id: DockingConnectorId,
) -> Option<&mut DockingClamp> {
    docking_clamps.iter_mut().find(|clamp| {
        clamp
            .connection()
            .map(|x| x.connector_id == connector_id)
            .unwrap_or(false)
    })
}

pub fn find_modules_with_capabilities_in_storages(
    storages: &[ModuleStorage],
    needed_capabilities: BTreeSet<ModuleCapability>,
    needed_primary_capabilities: BTreeSet<ModuleCapability>,
) -> Option<BTreeSet<ModuleId>> {
    for storage in storages {
        let mut needed_capabilities = needed_capabilities.clone();
        let mut needed_primary_capabilities = needed_primary_capabilities.clone();

        let mut modules: BTreeSet<ModuleId> = Default::default();
        for module in storage.iter() {
            let mut got_something: bool = false;
            for cap in module.capabilities() {
                if needed_capabilities.contains(cap) {
                    needed_capabilities.remove(cap);
                    got_something = true;
                }
            }

            for cap in module.primary_capabilities() {
                if needed_primary_capabilities.contains(cap) {
                    needed_primary_capabilities.remove(cap);
                    got_something = true;
                }
            }

            if got_something {
                modules.insert(module.id());
            }
        }

        if needed_capabilities.is_empty() && needed_primary_capabilities.is_empty() {
            return Some(modules);
        }
    }
    None
}

pub fn are_dockyard_components_suitable(
    storages: &[ModuleStorage],
    docking_clamps: &[DockingClamp],
    needed_capabilities: Vec<ModuleCapability>,
    needed_primary_capabilities: Vec<ModuleCapability>,
) -> bool {
    docking_clamps.len() > 0
        && docking_clamps.iter().any(|clamp| clamp.is_empty())
        && storages.iter().any(|storage| {
            (|| {
                let mut needed_capabilities = needed_capabilities.clone();
                for module in storage.iter() {
                    for cap in module.capabilities() {
                        if let Some(i) = needed_capabilities.iter().position(|x| *x == *cap) {
                            needed_capabilities.remove(i);
                        }
                    }
                }
                needed_capabilities.is_empty()
            })() && (|| {
                let mut needed_primary_capabilities = needed_primary_capabilities.clone();
                for module in storage.iter() {
                    for cap in module.primary_capabilities() {
                        if let Some(i) = needed_primary_capabilities.iter().position(|x| *x == *cap)
                        {
                            needed_primary_capabilities.remove(i);
                        }
                    }
                }
                needed_primary_capabilities.is_empty()
            })()
        })
}
