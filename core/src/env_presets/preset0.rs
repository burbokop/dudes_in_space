use std::rc::Rc;
use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::person::Person;
use rand::Rng;
use dudes_in_space_api::item::{Item, ItemStorage};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputRecipe};
use dudes_in_space_api::vessel::Vessel;
use crate::modules::{Assembler, DockyardFactory, PersonnelArea, ShuttleFactory};

pub fn new<R: Rng>(rng: &mut R) -> Environment {
    let person0 = Person::random(rng);
    let person1 = Person::random(rng);
    let person2 = Person::random(rng);

    let person0_id = person0.id();
    let spawn_station_personnel_area = PersonnelArea::new(vec![person0, person1, person2]);
    let spawn_station_assembler = Assembler::new(vec![
         AssemblyRecipe::new( vec![Item::new("steel".to_string(), 10)].try_into().unwrap(), Rc::new(ShuttleFactory)),
         AssemblyRecipe::new( vec![Item::new("steel".to_string(), 100)].try_into().unwrap(), Rc::new(DockyardFactory)),
    ], vec![Item::new("steel".to_string(), 10000000), Item::new("plastic".to_string(), 100000), Item::new("microelectronics".to_string(), 100)].try_into().unwrap());

    let spawn_station = Vessel::new(
        person0_id,
        (0., 0.).into(),
        vec![spawn_station_personnel_area, spawn_station_assembler],
    );

    Environment::new(vec![spawn_station],vec![])
}
