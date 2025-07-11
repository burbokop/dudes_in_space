use std::cell::RefMut;
use crate::bl::modules::{Module, ModuleCapability};

pub(crate) trait VesselPersonInterface {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>>;
}
