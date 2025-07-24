use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ModuleCapability {
    Cockpit,
    ModuleStorage,
    ItemStorage,
    FuelTank,
    Radar,
    Engine,
    DockingPort,
    Weapon,
    WarpDrive,
    Reactor,
    Crafting,
    Dockyard,
}
