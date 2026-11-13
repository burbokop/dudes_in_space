use crate::environment::Environment;
use crate::module::Module;
use crate::person::Person;
use crate::vessel::Vessel;

pub trait StatusCollector {
    fn enter_environment(&mut self, environment: &Environment);
    fn enter_vessel(&mut self, vessel: &Vessel);
    fn enter_module(&mut self, module: &dyn Module);
    fn enter_person(&mut self, person: &Person);

    fn exit_environment(&mut self);
    fn exit_vessel(&mut self);
    fn exit_module(&mut self);
    fn exit_person(&mut self);
}
