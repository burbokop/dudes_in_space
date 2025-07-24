use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::person::Person;
use rand::Rng;

pub(crate) fn new<R: Rng>(rng: &mut R) -> Environment {
    let person0 = Person::random(rng);
    let person1 = Person::random(rng);
    let person2 = Person::random(rng);

    // let spawn_station_personnel_area = PersonnelArea::new(vec![person0, person1, person2]);

    // let spawn_station = VesselCreateInfo {
    //     pos: (0., 0.).into(),
    //     modules: vec![spawn_station_personnel_area],
    // };

    // Environment::new(vec![spawn_station])

    Environment::new(vec![])
}
