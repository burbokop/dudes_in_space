use dudes_in_space_api::person::{Logger, PersonId, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogPiece {
    pub severity: Severity,
    pub message: String,
}

impl Display for LogPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.severity, self.message)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemLogger {
    #[serde(flatten)]
    data: BTreeMap<PersonId, Vec<LogPiece>>,
}

impl MemLogger {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn get(&self, person_id: &PersonId) -> &[LogPiece] {
        self.data
            .get(person_id)
            .map(|x| x.as_slice())
            .unwrap_or(&[])
    }
}

impl Logger for MemLogger {
    fn log(
        &mut self,
        person_id: &PersonId,
        person_name: &str,
        severity: Severity,
        message: String,
    ) {
        self.data
            .entry(*person_id)
            .or_default()
            .push(LogPiece { severity, message });
    }
}
