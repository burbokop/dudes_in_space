use crate::environment::{
    EnvironmentContext, EnvironmentRequest, RequestCreditLimitIncrease,
    RequestCreditLimitIncreaseResult,
};
use crate::finance::{Bank, BankRegistry, CurrencyGenerator};
use crate::module::ModuleConsole;
use crate::person::ThisPerson;
use crate::person::logger::PersonLogger;
use crate::utils::request::ReqContext;
use crate::vessel::VesselInternalConsole;
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use rand::prelude::SliceRandom;
use rand::{Rng, rng};
use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug, Eq, PartialEq)]
pub enum ObjectiveStatus<R> {
    /// Doing something
    InProgress,
    /// Waiting for some condition to be met
    Passive,
    // TODO: add Interrupted status
    Done(R),
}

pub trait Objective {
    type Result;
    type Error: Error + 'static;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<Self::Result>, Self::Error>;

    fn request_handler(&mut self) -> Option<Box<dyn ObjectiveRequestHandler>> {
        Some(Box::new(DefaultObjectiveRequestHandler))
    }
}

pub trait ObjectiveRequestHandler {
    fn handle_request_credit_limit_increase(
        &mut self,
        this_person: &mut ThisPerson,
        currency_generator: &CurrencyGenerator,
        bank_registry: &BankRegistry,
        req_context: &ReqContext,
        request: &mut EnvironmentRequest<
            RequestCreditLimitIncrease,
            RequestCreditLimitIncreaseResult,
        >,
    );
}

struct DefaultObjectiveRequestHandler;

impl<'a> ObjectiveRequestHandler for DefaultObjectiveRequestHandler {
    fn handle_request_credit_limit_increase(
        &mut self,
        this_person: &mut ThisPerson,
        currency_generator: &CurrencyGenerator,
        bank_registry: &BankRegistry,
        req_context: &ReqContext,
        request: &mut EnvironmentRequest<
            RequestCreditLimitIncrease,
            RequestCreditLimitIncreaseResult,
        >,
    ) {
        let this_person_wallet_id = this_person.finance.wallet().id().clone();
        this_person.finance.increase_credit_limit_or_create(
            request.input.new_limit,
            Bank::new(
                this_person.id.clone(),
                this_person_wallet_id,
                currency_generator.generate_name(&mut rng(), bank_registry, this_person),
            ),
        );

        request
            .promise
            .make_ready(
                req_context,
                RequestCreditLimitIncreaseResult::LimitIncreased,
            )
            .unwrap()
    }
}

pub trait DynObjective: Debug + Display + DynSerialize {
    fn pursue_dyn(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus<()>, Box<dyn Error>>;

    fn request_handler_dyn(&mut self) -> Option<Box<dyn ObjectiveRequestHandler>>;
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
    ) -> Result<ObjectiveStatus<()>, Box<dyn Error>> {
        match self.pursue(
            this_person,
            this_module,
            this_vessel,
            environment_context,
            logger,
        ) {
            Ok(ObjectiveStatus::InProgress) => Ok(ObjectiveStatus::InProgress),
            Ok(ObjectiveStatus::Passive) => Ok(ObjectiveStatus::Passive),
            Ok(ObjectiveStatus::Done(_)) => Ok(ObjectiveStatus::Done(())),
            Err(err) => Err(Box::new(err)),
        }
    }

    fn request_handler_dyn(&mut self) -> Option<Box<dyn ObjectiveRequestHandler>> {
        self.request_handler()
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
