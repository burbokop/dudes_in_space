use std::cell::RefCell;
use crate::bl::utils::math::{Point, Vector};
use crate::bl::{InputRecipe, Person, Recipe, Role};
use serde::de::{DeserializeOwned, Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use serde::ser::SerializeSeq;
use serde_intermediate::Intermediate;

pub(crate) trait Module: Debug {
    fn type_id(&self) -> String;
    fn serialize(&self) -> Intermediate;
    fn proceed(&mut self, v: & dyn VesselPersonInterface);
    fn accept_visitor(&self, v: &dyn ModuleVisitor<Result = ()>) -> Option<()>;
    fn capabilities(&self) -> &[ModuleCapability];
    fn recipes(&self) -> Vec<Recipe>;
    fn assembly_recipes(&self) -> Vec<AssemblyRecipe>;
}

pub(crate) trait ModuleSerializerDeserializer {
    fn type_id(&self) -> String;
    fn deserialize(&self, str: Intermediate) -> Result<Box<dyn Module>, String>;
}

pub(crate) struct ModuleSerializerDeserializerRegistry {
    data: BTreeMap<String, Box<dyn ModuleSerializerDeserializer>>,
}

impl ModuleSerializerDeserializerRegistry {
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub(crate) fn with<T: ModuleSerializerDeserializer + 'static>(mut self, sd: T) -> Self {
        let name = sd.type_id();
        self.data.insert(name, Box::new(sd));
        self
    }

    pub(crate) fn serialize<S: Serializer>(
        serializer: S,
        module: &RefCell< Box<dyn Module>>,
    ) -> Result<S::Ok, S::Error> {
        let module = module.borrow();
        
        let type_id = module.type_id();

        #[derive(Serialize)]
        struct Impl {
            tp: ModuleTypeId,
            payload: Intermediate,
        }
        
        Impl{ tp: type_id, payload: module.serialize() }.serialize( serializer)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(&self, deserializer: D) -> Result<Box<dyn Module>, D::Error> {
        #[derive(Deserialize)]
        struct Impl {
            tp: String,
            payload: Intermediate,
        }
        
        let i = Impl::deserialize(deserializer)?;
        
        let deser = self.data.get(&i.tp).expect(&format!("Module with id `{}` not found", i.tp));

        Ok(deser.deserialize(i.payload).map_err(|e| D::Error::custom(e))?)
    }
}

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