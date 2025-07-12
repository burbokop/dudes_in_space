use std::cell::RefCell;
use crate::bl::utils::math::{Point, Vector};
use crate::bl::{InputRecipe, Person, Recipe, Role};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use dudes_in_space_macro::dyn_serialize_trait;
use serde::de::Error as _;
use serde::ser::{Error, SerializeSeq};
use serde_intermediate::Intermediate;

pub(crate) trait Module: Debug + DynSerialize {
    fn proceed(&mut self, v: & dyn VesselPersonInterface);
    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()>;
    fn capabilities(&self) -> &[ModuleCapability];
    fn recipes(&self) -> Vec<Recipe>;
    fn assembly_recipes(&self) -> Vec<AssemblyRecipe>;
}

dyn_serialize_trait!(Module);

pub(crate) struct ModuleFactoryRegistry {
    
    data: BTreeMap<String, Box<dyn ModuleFactory>>,
}

pub(crate) type ModuleTypeId = String;

impl ModuleFactoryRegistry {
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
    
    pub(crate) fn with<T: ModuleFactory + 'static>(mut self, sd: T) -> Self {
        let name = sd.type_id();
        self.data.insert(name, Box::new(sd));
        self
    }

    fn assembly_recipe(&self, type_id: ModuleTypeId, input: InputRecipe) -> AssemblyRecipe {

        let f = self.data.get(&type_id).expect(&format!("Module with id `{}` not found", type_id));

        todo!()

        // AssemblyRecipe::new(input, f)
    }
}



pub(crate) trait ModuleVisitor {
    type Result;

    fn visit_personnel_area(&self, _: &PersonnelArea) -> Option<Self::Result>
    {
        None
    }

    fn visit_assembler(&self, _: &Assembler) -> Option<Self::Result>
    {
        None
    }

}


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
pub(crate) enum ModuleCapability {
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

mod personnel_area;
pub use personnel_area::*;
use crate::bl::utils::utils::Float;

mod assembler;
pub use assembler::*;

mod module_person_interfaces;
pub use module_person_interfaces::*;

mod vessel_person_interfaces;
pub use vessel_person_interfaces::*;

mod assembly_recipe;
pub use assembly_recipe::*;
use crate::bl::utils::dyn_serde::{DynDeserializeFactory, DynSerialize};