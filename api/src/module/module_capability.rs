use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ModuleCapability {
    ModuleStorage,
    ItemStorage,
    Dockyard,
    Crafting,
    PersonnelRoom,

    Cockpit,
    FuelTank,
    Radar,
    Engine,
    DockingPort,
    Weapon,
    WarpDrive,
    Reactor,
}
