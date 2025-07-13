use serde::ser::{Error, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

pub type PackageId = String;

pub trait Module: Debug + DynSerialize {
    fn package_id(&self) -> PackageId;
    fn proceed(&mut self, v: &dyn VesselPersonInterface);
    fn capabilities(&self) -> &[ModuleCapability];
    fn recipes(&self) -> Vec<Recipe>;
    fn assembly_recipes(&self) -> &[AssemblyRecipe];
}

dyn_serde_trait!(Module);

pub(crate) type ModuleTypeId = String;


pub(crate) struct Cockpit {}

impl CaptainControlPanel for Cockpit {}

pub(crate) struct Cargo {}
pub(crate) struct FuelTank {}
pub(crate) struct Radar {}

impl NavigatorControlPanel for Radar {}

pub(crate) struct Engine {}
pub(crate) struct DockingPort {}

pub(crate) struct ProjectileGun {}

impl GunnerControlPanel for ProjectileGun {}

pub(crate) struct MissileRack {}

impl GunnerControlPanel for MissileRack {}

pub(crate) struct WarpDrive {}
pub(crate) struct Radiators {}
pub(crate) struct Reactor {}
pub(crate) struct Workshop {}

impl WorkerControlPanel for Workshop {}

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ModuleCapability {
    Cockpit,
    Cargo,
    FuelTank,
    Radar,
    Engine,
    DockingPort,
    Weapon,
    WarpDrive,
    Reactor,
    Crafting,
}

mod module_person_interfaces;
pub use module_person_interfaces::*;

mod vessel_person_interfaces;
pub use vessel_person_interfaces::*;

mod assembly_recipe;
pub use assembly_recipe::*;
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use crate::items::Recipe;
