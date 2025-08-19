use crate::modules::{Assembler, PersonnelArea};
use dudes_in_space_api::environment::{Environment, Nebula};
use dudes_in_space_api::item::{ItemStack, ItemStorage, ItemVault};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::utils::physics::M3;
use dudes_in_space_api::vessel::Vessel;
use rand::Rng;

fn storage(item_vault: &ItemVault) -> ItemStorage {
    ItemStorage::from_vec(
        vec![
            ItemStack::new(item_vault, "steel".to_string(), 900000).unwrap(),
            ItemStack::new(item_vault, "plastic".to_string(), 90000).unwrap(),
            ItemStack::new(item_vault, "microelectronics".to_string(), 100).unwrap(),
        ],
        M3(1000000),
    )
    .unwrap()
}

fn station0<R: Rng>(rng: &mut R, item_vault: &ItemVault) -> Vessel {
    let person0 = Person::random(rng);
    let person1 = Person::random(rng);

    let person0_id = person0.id();
    let personnel_area = PersonnelArea::new(vec![person0, person1]);
    let assembler = Assembler::new(storage(item_vault));

    Vessel::new(
        "station0".to_string(),
        person0_id,
        (600., -300.).into(),
        vec![personnel_area, assembler],
    )
}

fn station1<R: Rng>(rng: &mut R, item_vault: &ItemVault) -> Vessel {
    let person0 = Person::random(rng);
    let person1 = Person::random(rng);
    let person2 = Person::random(rng);

    let person0_id = person0.id();
    let personnel_area = PersonnelArea::new(vec![person0, person1, person2]);
    let assembler = Assembler::new(storage(item_vault));

    Vessel::new(
        "station1".to_string(),
        person0_id,
        (-500., -500.).into(),
        vec![personnel_area, assembler],
    )
}

fn nebula() -> Nebula {
    Nebula::new(vec![
        (-200., -1000.).into(),
        (200., -1000.).into(),
        (100., 1000.).into(),
        (-300., 1000.).into(),
    ])
}

pub fn new<R: Rng>(rng: &mut R, item_vault: &ItemVault) -> Environment {
    Environment::new(
        vec![station0(rng, item_vault), station1(rng, item_vault)],
        vec![nebula()],
    )
}
