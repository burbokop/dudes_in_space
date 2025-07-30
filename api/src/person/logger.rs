use crate::person::PersonId;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => f.write_str("E"),
            Severity::Warning => f.write_str("W"),
            Severity::Info => f.write_str("I"),
        }
    }
}

pub trait Logger {
    fn log(&mut self, person_id: &PersonId, person_name: &str, severity: Severity, message: String);
}

pub struct PersonLogger<'id, 'name, 'l> {
    person_id: &'id PersonId,
    person_name: &'name str,
    logger: &'l mut dyn Logger,
}

impl<'id, 'name, 'l> PersonLogger<'id, 'name, 'l> {
    pub fn new(
        person_id: &'id PersonId,
        person_name: &'name str,
        logger: &'l mut dyn Logger,
    ) -> Self {
        Self {
            person_id,
            person_name,
            logger,
        }
    }
}

impl<'id, 'name, 'l> PersonLogger<'id, 'name, 'l> {
    pub fn info<M: ToString>(&mut self, message: M) {
        self.logger.log(
            self.person_id,
            self.person_name,
            Severity::Info,
            message.to_string(),
        )
    }

    pub fn warn<M: ToString>(&mut self, message: M) {
        self.logger.log(
            self.person_id,
            self.person_name,
            Severity::Warning,
            message.to_string(),
        )
    }

    pub fn err<M: ToString>(&mut self, message: M) {
        self.logger.log(
            self.person_id,
            self.person_name,
            Severity::Error,
            message.to_string(),
        )
    }
}
