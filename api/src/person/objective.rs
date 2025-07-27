use crate::module::{ModuleConsole, ProcessTokenContext};
use crate::person::{Awareness, Boldness, Gender, Morale, Passion, PersonId};
use crate::vessel::VesselConsole;
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use rand::Rng;
use rand::prelude::SliceRandom;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use crate::person::logger::PersonLogger;

#[derive(Debug, Eq, PartialEq)]
pub enum ObjectiveStatus {
    InProgress,
    Done,
}

pub trait Objective {
    type Error: Error + 'static;
    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error>;
}

pub trait DynObjective: Debug + DynSerialize {
    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
    ) -> Result<ObjectiveStatus, Box<dyn Error>>;
}

dyn_serde_trait!(DynObjective, ObjectiveSeed);

impl<T: Objective + Debug + DynSerialize> DynObjective for T {
    fn pursue(
        &mut self,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselConsole,
        process_token_context: &ProcessTokenContext,
        logger: PersonLogger,
    ) -> Result<ObjectiveStatus, Box<dyn Error>> {
        Ok(self
            .pursue(this_module, this_vessel, process_token_context, logger)
            .map_err(|e| Box::new(e))?)
    }
}

pub trait ObjectiveDecider {
    fn consider(
        &self,
        person_id: PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
    ) -> Option<Box<dyn DynObjective>>;
}

#[derive(Default)]
pub struct ObjectiveDeciderVault {
    data: Vec<Box<dyn ObjectiveDecider>>,
}

impl ObjectiveDeciderVault {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn decide<R: Rng>(
        &self,
        rng: &mut R,
        person_id: PersonId,
        age: u8,
        gender: Gender,
        passions: &[Passion],
        morale: Morale,
        boldness: Boldness,
        awareness: Awareness,
    ) -> Option<Box<dyn DynObjective>> {
        let mut data: Vec<&dyn ObjectiveDecider> = self.data.iter().map(|x| x.as_ref()).collect();
        data.shuffle(rng);
        data.into_iter()
            .find_map(|x| x.consider(person_id ,age, gender, passions, morale, boldness, awareness))
    }

    pub fn with<T: ObjectiveDecider + 'static>(mut self, decider: T) -> Self {
        self.data.push(Box::new(decider));
        self
    }
}
