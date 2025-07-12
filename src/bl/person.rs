use crate::bl::PersonObjective::CraftingVessels;
use crate::bl::modules::{Module, ModuleCapability, VesselPersonInterface};
use rand::distr::StandardUniform;
use rand::prelude::{Distribution, IndexedRandom, IteratorRandom, SliceRandom};
use rand::{Rng, rng};
use sdl2::libc::printf;
use serde::{Deserialize, Serialize};
use std::cell::RefMut;
use std::collections::BTreeSet;

fn random_name<R: Rng>(rng: &mut R, gender: Gender) -> String {
    let male_names = [
        "Tyler",
        "Yurem",
        "Justus",
        "Kane",
        "Maximillian",
        "Mario",
        "Chaim",
        "Braxton",
        "Devon",
        "Noel",
        "Ezekiel",
        "Samir",
        "Jayden",
        "Andrew",
        "Drew",
        "Alden",
    ];

    let female_names = [
        "Olivia",
        "Sydney",
        "Jakayla",
        "Tabitha",
        "Janessa",
        "Krista",
        "Madeline",
        "Janelle",
        "Kennedy",
        "Melissa",
        "Kamila",
        "Shannon",
        "Mariana",
        "Lizeth",
        "Elizabeth",
        "Dana",
    ];

    match gender {
        Gender::CisMale => male_names.choose(rng).unwrap().to_string(),
        Gender::CisFemale => female_names.choose(rng).unwrap().to_string(),
        Gender::MTFTrans => female_names.choose(rng).unwrap().to_string(),
        Gender::FTMTrans => male_names.choose(rng).unwrap().to_string(),
        Gender::NonBinary => male_names
            .iter()
            .chain(female_names.iter())
            .choose(rng)
            .unwrap()
            .to_string(),
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Copy, Clone)]
pub(crate) enum Passion {
    Trade,
    Crafting,
    Adventuring,
    Flying,
    Ruling,
    Money,
    Drugs,
    Sex,
}

impl Distribution<Passion> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Passion {
        match rng.random_range(0..7) {
            0 => Passion::Trade,
            1 => Passion::Adventuring,
            2 => Passion::Flying,
            3 => Passion::Ruling,
            4 => Passion::Money,
            5 => Passion::Drugs,
            6 => Passion::Sex,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Morale {
    SickBastard,
    Mercantile,
    TheEndJustifiesTheMeans,
    TitForTat,
    Altruist,
    Saint,
}

impl Distribution<Morale> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Morale {
        match rng.random_range(0..6) {
            0 => Morale::SickBastard,
            1 => Morale::Mercantile,
            2 => Morale::TheEndJustifiesTheMeans,
            3 => Morale::TitForTat,
            4 => Morale::Altruist,
            5 => Morale::Saint,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Boldness {
    PantsShittingWorm,
    Unconfident,
    Cautious,
    Average,
    Brave,
    WithoutSelfPreservation,
}

impl Distribution<Boldness> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Boldness {
        match rng.random_range(0..6) {
            0 => Boldness::PantsShittingWorm,
            1 => Boldness::Unconfident,
            2 => Boldness::Cautious,
            3 => Boldness::Average,
            4 => Boldness::Brave,
            5 => Boldness::WithoutSelfPreservation,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Awareness {
    Monkey,
    TrumpSupporter,
    Dummy,
    Average,
    Perceptive,
    Ascended,
}

impl Distribution<Awareness> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Awareness {
        match rng.random_range(0..6) {
            0 => Awareness::Monkey,
            1 => Awareness::TrumpSupporter,
            2 => Awareness::Dummy,
            3 => Awareness::Average,
            4 => Awareness::Perceptive,
            5 => Awareness::Ascended,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Role {
    Captain,
    Navigator,
    Gunner,
    Worker,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum Gender {
    CisMale,
    CisFemale,
    MTFTrans,
    FTMTrans,
    NonBinary,
}

impl Distribution<Gender> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gender {
        match rng.random_range(0..5) {
            0 => Gender::CisMale,
            1 => Gender::CisFemale,
            2 => Gender::MTFTrans,
            3 => Gender::FTMTrans,
            4 => Gender::NonBinary,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub(crate) enum PersonObjective {
    CraftingVessels,
}

#[derive(Debug, Serialize, Deserialize)]
enum PersonState {
    Idle,
    PursuingObjective(PersonObjective),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Person {
    name: String,
    age: u8,
    gender: Gender,
    passions: Vec<Passion>,
    morale: Morale,
    boldness: Boldness,
    awareness: Awareness,
    state: PersonState,
}

impl Person {
    pub(crate) fn random<R: Rng>(rng: &mut R) -> Self {
        let gender = rng.random();
        Self {
            name: random_name(rng, gender),
            age: rng.random_range(15..=80),
            gender,
            passions: (0..rng.random_range(0..=4))
                .map(|_| rng.random())
                .collect::<BTreeSet<Passion>>()
                .into_iter()
                .collect(),
            morale: rng.random(),
            boldness: rng.random(),
            awareness: rng.random(),
            state: PersonState::Idle,
        }
    }

    pub(crate) fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        match &self.state {
            PersonState::Idle => self.decide_objective(),
            PersonState::PursuingObjective(objective) => self.pursue_objective(v, *objective),
        }
    }

    fn decide_objective(&mut self) {
        {
            let mut passions = self.passions.clone();
            passions.shuffle(&mut rand::rng());

            while let Some(next) = passions.pop() {
                if match next {
                    Passion::Trade => false,
                    Passion::Crafting => false,
                    Passion::Adventuring => false,
                    Passion::Flying => false,
                    Passion::Ruling => false,
                    Passion::Money => false,
                    Passion::Drugs => false,
                    Passion::Sex => false,
                } {
                    break;
                }
            }
        }

        // Fallback choice
        {
            let mut passions = self.passions.clone();
            passions.shuffle(&mut rand::rng());

            while let Some(next) = passions.pop() {
                if match next {
                    Passion::Trade => false,
                    Passion::Crafting => false,
                    Passion::Adventuring => false,
                    Passion::Flying => {
                        self.state = PersonState::PursuingObjective(CraftingVessels);
                        true
                    }
                    Passion::Ruling => false,
                    Passion::Money => false,
                    Passion::Drugs => false,
                    Passion::Sex => false,
                } {
                    break;
                }
            }
        }
    }

    fn pursue_objective(&mut self, v: &dyn VesselPersonInterface, objective: PersonObjective) {
        match objective {
            CraftingVessels => {
                let is_crafting_module_suitable = |crafting_module: &Box<dyn Module>,
                                                   mut needed_caps: Vec<ModuleCapability>|
                 -> bool {
                    for r in crafting_module.assembly_recipes() {
                        for cap in r.output_capabilities() {
                            if let Some(i) = needed_caps.element_offset(&cap) {
                                needed_caps.remove(i);
                            }
                        }
                    }
                    needed_caps.is_empty()
                };

                let needed_caps = vec![
                    ModuleCapability::Cockpit,
                    ModuleCapability::Engine,
                    ModuleCapability::Reactor,
                    ModuleCapability::FuelTank,
                ];

                for crafting_module in v.modules_with_cap(ModuleCapability::Crafting) {
                    if is_crafting_module_suitable(&crafting_module, needed_caps.clone()) {
                        println!("move to module: {:?}", crafting_module.type_id());
                        return;
                    }
                }
                self.state = PersonState::Idle;
            }
        }
    }
}
