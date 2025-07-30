use crate::module::{
       ModuleConsole, ProcessTokenContext,
};
use crate::person::logger::{Logger, PersonLogger};
use crate::person::objective::{ ObjectiveSeed, ObjectiveStatus};
use crate::person::{DynObjective, ObjectiveDeciderVault, Severity};
use crate::utils::tagged_option::TaggedOptionSeed;
use crate::vessel::VesselConsole;
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use rand::Rng;
use rand::distr::StandardUniform;
use rand::prelude::{Distribution, IndexedRandom, IteratorRandom};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::{NonNilUuid, Uuid};

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
pub enum Passion {
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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Morale {
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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Boldness {
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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Awareness {
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
pub enum Gender {
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

pub type PersonId = NonNilUuid;

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::person::PersonSeed::<'v>)]
pub struct Person {
    id: PersonId,
    name: String,
    age: u8,
    gender: Gender,
    passions: Vec<Passion>,
    morale: Morale,
    boldness: Boldness,
    awareness: Awareness,
    #[serde(with = "crate::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.objective_seed)]
    objective: Option<Box<dyn DynObjective>>,
}

#[derive(Clone)]
pub struct PersonSeed<'v> {
    objective_seed: TaggedOptionSeed<ObjectiveSeed<'v>>,
}

impl<'v> PersonSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn DynObjective>) -> Self {
        Self {
            objective_seed: TaggedOptionSeed::new(ObjectiveSeed::new(vault)),
        }
    }
}

impl Person {
    pub fn id(&self) -> PersonId {
        self.id
    }

    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let gender = rng.random();
        Self {
            id: NonNilUuid::new(Uuid::new_v4()).unwrap(),
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
            objective: None,
        }
    }

    pub fn proceed<R: Rng>(
        &mut self,
        rng: &mut R,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        match &mut self.objective {
            None => {
                self.objective = decider_vault.decide(
                    rng,
                    self.id,
                    self.age,
                    self.gender,
                    &self.passions,
                    self.morale,
                    self.boldness,
                    self.awareness,
                )
            }
            Some(objective) => {
                match objective.pursue_dyn(
                    this_module,
                    this_vessel,
                    process_token_context,
                    &mut PersonLogger::new(&self.id, &self.name, logger),
                ) {
                    Ok(ObjectiveStatus::InProgress) => {}
                    Ok(ObjectiveStatus::Done) => self.objective = None,
                    Err(err) => {
                        logger.log(
                            &self.id,
                            &self.name,
                            Severity::Error,
                            format!("{} failed: {}", objective.type_id(), err),
                        );
                        self.objective = None
                    }
                }
            }
        }
    }
}
