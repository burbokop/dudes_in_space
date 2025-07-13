use std::cell::RefMut;
use crate::modules::{Module, ModuleCapability};

pub trait VesselPersonInterface {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>>;
}
