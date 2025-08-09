use crate::item::{ ItemVolume};
use crate::module::{
    ConcatModuleCapabilities, Module, ModuleCapability, ModuleConsole, ModuleId, ModuleStorage,
};
use crate::vessel::{
    DockingClamp, DockingClampConnection, DockingConnectorId, VesselConsole, VesselId,
};
use std::collections::BTreeSet;
use std::ops::{Deref, Try};
use crate::utils::physics::M3;

pub fn this_vessel_caps(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
) -> BTreeSet<ModuleCapability> {
    this_vessel
        .capabilities()
        .concat(this_module.capabilities())
}

pub fn this_vessel_primary_caps(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
) -> BTreeSet<ModuleCapability> {
    this_vessel
        .primary_capabilities()
        .concat(this_module.primary_capabilities())
}

pub fn this_vessel_has_caps(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
    needed_caps: impl IntoIterator<Item = ModuleCapability>,
) -> bool {
    let this_vessel_caps = this_vessel_caps(this_module, this_vessel);
    needed_caps
        .into_iter()
        .all(|cap| this_vessel_caps.contains(&cap))
}

pub fn this_vessel_has_primary_caps(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
    needed_caps: impl IntoIterator<Item = ModuleCapability>,
) -> bool {
    let this_vessel_caps = this_vessel_primary_caps(this_module, this_vessel);
    needed_caps
        .into_iter()
        .all(|cap| this_vessel_caps.contains(&cap))
}

pub struct ForEachDockingClampsEntry<'d, 'm> {
    pub clamp: &'d DockingClamp,
    pub module: Option<&'m dyn Module>,
}

pub fn for_each_docking_clamps_with_vessel_which_has_caps<F, R>(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
    caps: &[ModuleCapability],
    primary_caps: &[ModuleCapability],
    f: F,
) -> R
where
    F: FnMut(ForEachDockingClampsEntry) -> R,
    R: Try<Output = ()>,
{
    this_vessel
        .modules_with_capability(ModuleCapability::DockingClamp)
        .iter()
        .map(|m| {
            m.docking_clamps()
                .iter()
                .map(|x| (x, Some(m.deref().deref())))
        })
        .flatten()
        .chain(this_module.docking_clamps().iter().map(|x| (x, None)))
        .filter_map(|(clamp, module)| {
            clamp.connection().and_then(
                |DockingClampConnection {
                     vessel,
                     connector_id,
                 }| {
                    let vessel_capabilities = vessel.capabilities();
                    let vessel_primary_capabilities = vessel.primary_capabilities();
                    (caps.iter().all(|cap| vessel_capabilities.contains(&cap))
                        && primary_caps
                            .iter()
                            .all(|cap| vessel_primary_capabilities.contains(&cap)))
                    .then_some(ForEachDockingClampsEntry { clamp, module })
                },
            )
        })
        .try_for_each(f)
}

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

pub fn total_primary_free_space(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
) -> ItemVolume {
    this_vessel
        .modules_with_primary_capability(ModuleCapability::ItemStorage)
        .iter()
        .map(|module| {
            module
                .storages()
                .iter()
                .map(|storage| storage.free_space())
                .sum::<ItemVolume>()
        })
        .sum::<ItemVolume>()
        + if this_module
            .primary_capabilities()
            .contains(&ModuleCapability::ItemStorage)
        {
            this_module
                .storages()
                .iter()
                .map(|storage| storage.free_space())
                .sum::<ItemVolume>()
        } else {
            M3(0)
        }
}
