use crate::module::{ConcatModuleCapabilities, ModuleCapability, ModuleConsole};
use crate::vessel::{DockingClamp, VesselConsole};
use std::cell::{Ref, RefMut};

fn can_this_vessel_fly(
    this_module: &mut dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
) -> bool {
    let this_vessel_caps = this_vessel
        .capabilities()
        .concat(this_module.capabilities());
    let needed_caps = vec![
        ModuleCapability::Cockpit,
        ModuleCapability::Engine,
        ModuleCapability::Reactor,
        ModuleCapability::FuelTank,
    ];

    if needed_caps.iter().all(|cap| this_vessel_caps.contains(cap)) {
        true
    } else {
        false
    }
}

fn for_each_docking_clamps_with_vessels_which_can_fly<F>(
    this_module: &dyn ModuleConsole,
    this_vessel: &dyn VesselConsole,
    f: F,
) where
    F: FnMut(&DockingClamp),
{
    this_vessel
        .modules_with_cap(ModuleCapability::DockingClamp)
        .iter()
        .map(|m| m.docking_clamps().iter())
        .flatten()
        .chain(this_module.docking_clamps().iter())
        .filter_map(|clamp| {
            clamp.vessel_docked().and_then(|vessel| {
                let vessel_caps = vessel.capabilities();

                [
                    ModuleCapability::Cockpit,
                    ModuleCapability::Engine,
                    ModuleCapability::Reactor,
                    ModuleCapability::FuelTank,
                ]
                .iter()
                .all(|cap| vessel_caps.contains(cap))
                .then_some(clamp)
            })
        })
        .for_each(f)
}
