use crate::modules::{Assembler, DockyardFactory, PersonnelArea, ShuttleFactory, UnmannedTradingTerminal};
use dudes_in_space_api::environment::{Environment, Nebula};
use dudes_in_space_api::item::{Item, ItemStorage};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::recipe::AssemblyRecipe;
use dudes_in_space_api::vessel::Vessel;
use rand::Rng;
use std::rc::Rc;

fn recipes() -> Vec<AssemblyRecipe> { vec![
    AssemblyRecipe::new(
        vec![Item::new("steel".to_string(), 10)].try_into().unwrap(),
        Rc::new(ShuttleFactory {}),
    ),
    AssemblyRecipe::new(
        vec![Item::new("steel".to_string(), 100)]
            .try_into()
            .unwrap(),
        Rc::new(DockyardFactory {}),
    ),
]}

fn storage() -> ItemStorage {
    ItemStorage::try_from_vec(vec![
        Item::new("steel".to_string(),   900000),
        Item::new("plastic".to_string(), 90000),
        Item::new("microelectronics".to_string(), 100),
    ]
                              ,1000000)
        .unwrap()
}

fn station0<R: Rng>(rng: &mut R) -> Vessel {
    let person0 = Person::random(rng);
    let person1 = Person::random(rng);

    let person0_id = person0.id();
    let personnel_area = PersonnelArea::new(vec![person0, person1]);
    let assembler = Assembler::new(
        recipes(),
        storage(),
    );
    let trading_terminal = UnmannedTradingTerminal::new();

    Vessel::new(person0_id, (600., -300.).into(), vec![personnel_area, assembler, trading_terminal],)
}

fn station1<R: Rng>(rng: &mut R) -> Vessel {
    let person0 = Person::random(rng);
    let person1 = Person::random(rng);
    let person2 = Person::random(rng);

    let person0_id = person0.id();
    let personnel_area = PersonnelArea::new(vec![person0, person1, person2]);
    let assembler = Assembler::new(
        recipes(),
        storage(),
    );
    let trading_terminal = UnmannedTradingTerminal::new();

    Vessel::new(
        person0_id,
        (-500., -500.).into(),
        vec![personnel_area, assembler, trading_terminal],
    )
}

fn nebula() -> Nebula {
    Nebula::new(vec![
        (-200.,-1000.).into(),
        (200.,-1000.).into(),
        (100.,1000.).into(),
        (-300.,1000.).into(),
    ])
}

pub fn new<R: Rng>(rng: &mut R) -> Environment {
    Environment::new(vec![station0(rng), station1(rng)], vec![nebula()])
}
