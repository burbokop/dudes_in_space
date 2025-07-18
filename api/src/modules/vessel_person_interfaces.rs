use crate::modules::{Module, ModuleCapability, ModuleId};
use std::cell::RefMut;
use crate::Person;

pub trait VesselPersonInterface {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>>;
    fn move_to_module(&self, person: &Person, id: ModuleId);
}
