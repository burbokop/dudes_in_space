use std::fmt::Display;
use crate::person::PersonId;

pub enum Severity {
    Error,
    Warning,
    Info,
}

pub trait Logger {
    fn log(&mut self, person: &PersonId, severity: Severity, message: String);
}

pub struct PersonLogger<'id, 'l> {
    person_id: &'id PersonId,
    logger: &'l mut dyn Logger,
}

impl<'id, 'l> PersonLogger<'id, 'l> {
    pub fn new(person_id: &'id PersonId, logger: &'l mut dyn Logger) -> Self {
        Self { person_id, logger }
    }
}

impl<'id, 'l> PersonLogger<'id, 'l> {
    pub fn log<M: ToString>(&mut self, severity: Severity, message: M) {
        self.logger.log(self.person_id, severity, message.to_string())
    }
}
