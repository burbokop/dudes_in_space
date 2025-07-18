use crate::modules::{ Module, ModuleCapability, ModuleId, VesselPersonInterface};
use crate::person::CraftingVesselsStage::MovingToCraftingModule;
use crate::person::PersonObjective::CraftingVessels;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "tp")]
pub(crate) enum CraftingVesselsStage {
    SearchingForCraftingModule,
    MovingToCraftingModule { dst: ModuleId },
    Crufting,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CraftingVesselsObjective {
    stage: CraftingVesselsStage
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TradingObjective {
    i: Option<u8>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "objective_tp")]
pub(crate) enum PersonObjective {
    CraftingVessels(CraftingVesselsObjective),
    Trading(TradingObjective),
}

#[derive(Debug, Serialize, Deserialize)]
struct PersonStateIdle {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tp")]
enum PersonState {
    Idle(PersonStateIdle),
    PursuingObjective(PersonObjective),
}

impl Default for PersonState {
    fn default() -> Self {
        Self::Idle(PersonStateIdle{})
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
            state: PersonState::Idle(PersonStateIdle{}),
        }
    }

    pub fn proceed(&mut self, v: &dyn VesselPersonInterface) {
        match &self.state {
            PersonState::Idle (_) => self.decide_objective(),
            PersonState::PursuingObjective(objective) => {
                self.pursue_objective(v, objective.clone())
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
                        println!("move to vessel");
                        self.state = PersonState::PursuingObjective(CraftingVessels(CraftingVesselsObjective {
                            stage: CraftingVesselsStage::SearchingForCraftingModule {},
                        }));
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

    fn pursue_objective(&mut self, v: & dyn VesselPersonInterface, objective: PersonObjective) {
        match objective {
            CraftingVessels(objective) => match objective.stage {
                CraftingVesselsStage::SearchingForCraftingModule {} => {
                    let is_crafting_module_suitable = |crafting_module: &Box<dyn Module>,
                                                       mut needed_caps: Vec<ModuleCapability>|
                     -> bool {
                        for r in crafting_module.assembly_recipes() {
                            for cap in r.output_capabilities() {
                                if let Some(i) = needed_caps.iter().position(|x| *x == *cap) {
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
                            self.state = PersonState::PursuingObjective(CraftingVessels(CraftingVesselsObjective {
                                stage: MovingToCraftingModule {
                                    dst: crafting_module.id(),
                                },
                            }));
                            return;
                        }
                    }
                    self.state = PersonState::Idle(PersonStateIdle{});
                }
                CraftingVesselsStage::MovingToCraftingModule { dst } => {
                    v.move_to_module(self, dst);
                }
                CraftingVesselsStage::Crufting {} => {
                    todo!()
                }
            },
            PersonObjective::Trading(_) => todo!(),
        }
    }
}
