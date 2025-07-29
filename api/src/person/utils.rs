use std::collections::BTreeSet;
use std::ops::{Deref, Try};
use crate::module::{ConcatModuleCapabilities, Module, ModuleCapability, ModuleConsole};
use crate::vessel::{DockingClamp, VesselConsole};

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
    mut needed_caps: impl IntoIterator<Item = ModuleCapability>,
) -> bool {
    let this_vessel_caps = this_vessel_caps(this_module, this_vessel);
    needed_caps.into_iter().all(|cap| this_vessel_caps.contains(&cap))
}

pub fn this_vessel_has_primary_caps(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
    mut needed_caps: impl IntoIterator<Item = ModuleCapability>,
) -> bool {
    let this_vessel_caps= this_vessel_primary_caps(this_module, this_vessel);
    needed_caps.into_iter().all(|cap| this_vessel_caps.contains(&cap))
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
) -> R where
    F: FnMut(ForEachDockingClampsEntry) -> R,
    R: Try<Output = ()>
{
    this_vessel
        .modules_with_cap(ModuleCapability::DockingClamp)
        .iter()
        .map(|m| m.docking_clamps().iter().map(|x|(x, Some(m.deref().deref()))))
        .flatten()
        .chain(this_module.docking_clamps().iter().map(|x|(x, None)))
        .filter_map(|(clamp, module)| {
            clamp.vessel_docked().and_then(|vessel| {
                let vessel_capabilities = vessel.capabilities();
                let vessel_primary_capabilities = vessel.primary_capabilities();
                (caps.iter()
                .all(|cap| vessel_capabilities.contains(&cap))
                    && primary_caps.iter()
                .all(|cap| vessel_primary_capabilities.contains(&cap)))
                .then_some(ForEachDockingClampsEntry { clamp, module })
            })
        })
        .try_for_each(f)
}
