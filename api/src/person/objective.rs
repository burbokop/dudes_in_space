use crate::environment::EnvironmentContext;
use crate::module::ModuleConsole;
use crate::person::logger::PersonLogger;
use crate::person::{Awareness, Boldness, Gender, Money, Morale, Passion, PersonId};
use crate::vessel::VesselInternalConsole;
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use rand::Rng;
use rand::prelude::SliceRandom;
use std::error::Error;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq)]
pub enum ObjectiveStatus {
    InProgress,
    Done,
}

pub struct PersonInfo<'a> {
    pub id: &'a PersonId,
    pub age: &'a u8,
    pub gender: &'a Gender,
    pub passions: &'a [Passion],
    pub morale: &'a Morale,
    pub boldness: &'a Boldness,
    pub awareness: &'a Awareness,
    pub budget: &'a Money,
}

pub trait Objective {
    type Error: Error + 'static;
    fn pursue(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error>;
}

pub trait DynObjective: Debug + DynSerialize {
    fn pursue_dyn(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Box<dyn Error>>;
}

dyn_serde_trait!(DynObjective, ObjectiveSeed);

impl<T: Objective + Debug + DynSerialize> DynObjective for T {
    fn pursue_dyn(
        &mut self,
        this_person: &PersonInfo,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Box<dyn Error>> {
        Ok(self
            .pursue(
                this_person,
                this_module,
                this_vessel,
                environment_context,
                logger,
            )
            .map_err(|e| Box::new(e))?)
    }
}

pub trait ObjectiveDecider {
    fn consider(
        &self,
        person: &PersonInfo,
        logger: &mut PersonLogger,
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
        person: &PersonInfo,
        logger: &mut PersonLogger,
    ) -> Option<Box<dyn DynObjective>> {
        let mut data: Vec<&dyn ObjectiveDecider> = self.data.iter().map(|x| x.as_ref()).collect();
        data.shuffle(rng);
        data.into_iter().find_map(|x| x.consider(person, logger))
    }

    pub fn with<T: ObjectiveDecider + 'static>(mut self, decider: T) -> Self {
        self.data.push(Box::new(decider));
        self
    }
}
