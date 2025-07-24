use crate::module::{ModuleCapability, ModuleConsole};
use crate::person::crafting_vessels_objective::CraftingVesselsObjective;
use crate::person::objective::{Objective, ObjectiveStatus};
use crate::vessel::VesselConsole;
use rand::Rng;
use rand::distr::StandardUniform;
use rand::prelude::{Distribution, IndexedRandom, IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tp")]
enum PersonState {
    Idle,
    PursuingObjective(Objective),
}

impl Default for PersonState {
    fn default() -> Self {
        Self::Idle
    }
}

pub type PersonId = Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    id: PersonId,
    name: String,
    age: u8,
    gender: Gender,
    passions: Vec<Passion>,
    morale: Morale,
    boldness: Boldness,
    awareness: Awareness,
    #[serde(default)]
    state: PersonState,
}

impl Person {
    pub fn id(&self) -> PersonId {
        self.id
    }

    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let gender = rng.random();
        Self {
            id: Uuid::new_v4(),
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

    pub fn proceed(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
    ) {
        match &mut self.state {
            PersonState::Idle => self.decide_objective(),
            PersonState::PursuingObjective(objective) => {
                match objective.pursue(self.id, this_module, this_vessel) {
                    Ok(ObjectiveStatus::InProgress) => {}
                    Ok(ObjectiveStatus::Done) => todo!(),
                    Err(err) => {
                        eprintln!(
                            "Objective performed by person {} ({}) failed: {}",
                            self.id, self.name, err
                        );
                        self.state = PersonState::Idle
                    }
                }
            }
        }
    }

    fn decide_objective(&mut self) {
        println!("decide objective: {} -> {:?}", self.name, self.passions);

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
                        let needed_caps = vec![
                            ModuleCapability::Cockpit,
                            ModuleCapability::Engine,
                            ModuleCapability::Reactor,
                            ModuleCapability::FuelTank,
                        ];

                        self.state = PersonState::PursuingObjective(Objective::CraftingVessels(
                            CraftingVesselsObjective::new(needed_caps),
                        ));
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
}
