use crate::bl::modules::{Module, ModuleCapability};
use std::cell::RefMut;

pub(crate) trait VesselPersonInterface {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>>;
}
