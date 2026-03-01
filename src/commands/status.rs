use clap::Parser;
use colored::Colorize;
use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::module::Module;
use dudes_in_space_api::person::{Person, StatusCollector};
use dudes_in_space_api::vessel::Vessel;

static TAB: &str = "   ";

#[derive(Parser)]
pub(crate) struct StatusCommand {
    #[arg(short = 'p', long)]
    with_passions: bool,
}

impl StatusCommand {
    pub(crate) fn exec(self, environment: &Environment) {
        environment.collect_status(&mut StdOutStatusCollector::new(self.with_passions));
    }
}

struct StdOutStatusCollector {
    with_passions: bool,
    level: usize,
}

impl StdOutStatusCollector {
    fn new(with_passions: bool) -> Self {
        Self {
            with_passions,
            level: 0,
        }
    }
}

impl StatusCollector for StdOutStatusCollector {
    fn enter_environment(&mut self, environment: &Environment) {
        println!("vessels:");
        self.level += 1
    }

    fn enter_vessel(&mut self, vessel: &Vessel) {
        println!("{}{}:", TAB.repeat(self.level), vessel.id());
        self.level += 1
    }

    fn enter_module(&mut self, module: &dyn Module) {
        println!("{}{}:", TAB.repeat(self.level), module.type_id());
        self.level += 1
    }

    fn enter_person(&mut self, person: &Person) {
        if self.with_passions {
            println!(
                "{} (passions: {:?})",
                format!(
                    "{}{} -> {}",
                    TAB.repeat(self.level),
                    person.name(),
                    person.objective_type_id().unwrap_or("None".to_string()),
                )
                .green(),
                person.passions(),
            );
        } else {
            println!(
                "{}",
                format!(
                    "{}{} -> {}",
                    TAB.repeat(self.level),
                    person.name(),
                    person.objective_type_id().unwrap_or("None".to_string())
                )
                .green()
            );
        }
        self.level += 1
    }

    fn exit_environment(&mut self) {
        self.level -= 1;
    }

    fn exit_vessel(&mut self) {
        self.level -= 1;
    }

    fn exit_module(&mut self) {
        self.level -= 1;
    }

    fn exit_person(&mut self) {
        self.level -= 1;
    }
}
