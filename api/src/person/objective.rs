use crate::environment::{
    EnvironmentContext, EnvironmentRequest, RequestCreditLimitIncrease,
    RequestCreditLimitIncreaseResult,
};
use crate::module::ModuleConsole;
use crate::person::ThisPerson;
use crate::person::logger::PersonLogger;
use crate::vessel::VesselInternalConsole;
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use rand::Rng;
use rand::prelude::SliceRandom;
use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug, Eq, PartialEq)]
pub enum ObjectiveStatus {
    InProgress,
    Done,
}

pub trait Objective {
    type Error: Error + 'static;
    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error>;
}

pub trait ObjectiveRequestHandler {
    fn handle_request_credit_limit_increase(
        &mut self,
        request: &mut EnvironmentRequest<
            RequestCreditLimitIncrease,
            RequestCreditLimitIncreaseResult,
        >,
    );
}

pub trait DynObjective: Debug + Display + DynSerialize {
    fn pursue_dyn(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Box<dyn Error>>;

    fn request_handler_dyn(&mut self) -> Option<&mut dyn ObjectiveRequestHandler>;
}

dyn_serde_trait!(DynObjective, ObjectiveSeed);

impl<T: Objective + Debug + Display + DynSerialize> DynObjective for T {
    fn pursue_dyn(
        &mut self,
        this_person: &mut ThisPerson,
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

    fn request_handler_dyn(&mut self) -> Option<&mut dyn ObjectiveRequestHandler> {
        todo!()
    }
}

pub trait ObjectiveDecider {
    fn consider(
        &self,
        person: &ThisPerson,
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
        person: &ThisPerson,
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
