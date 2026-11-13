use dudes_in_space_api::person::{Logger, PersonId, Severity};

pub(crate) struct StdOutLogger;

impl Logger for StdOutLogger {
    fn log(
        &mut self,
        person_id: &PersonId,
        person_name: &str,
        severity: Severity,
        message: String,
    ) {
        match severity {
            Severity::Error => {
                eprintln!("{} {} ({}): {}", severity, person_id, person_name, message)
            }
            Severity::Warning => {
                eprintln!("{} {} ({}): {}", severity, person_id, person_name, message)
            }
            Severity::Info => println!("{} {} ({}): {}", severity, person_id, person_name, message),
        }
    }
}
