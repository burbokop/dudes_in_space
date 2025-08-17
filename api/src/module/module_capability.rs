use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ModuleCapability {
    ModuleStorage,
    ItemStorage,
    Dockyard,
    ModuleCrafting,
    ItemConsumption,
    ItemProduction,
    ItemCrafting,
    PersonnelRoom,
    DockingClamp,
    DockingConnector,
    TradingTerminal,
    VesselSellingTerminal,

    Cockpit,
    FuelTank,
    Radar,
    Engine,
    Weapon,
    WarpDrive,
    Reactor,
}

pub trait ConcatModuleCapabilities<Rhs> {
    type Output;
    fn concat(self, rhs: Rhs) -> Self::Output;
}

impl ConcatModuleCapabilities<&[ModuleCapability]> for BTreeSet<ModuleCapability> {
    type Output = Self;

    fn concat(mut self, rhs: &[ModuleCapability]) -> Self::Output {
        for rhs in rhs {
            self.insert(*rhs);
        }
        self
    }
}
