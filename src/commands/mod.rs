mod status;
mod step;

use crate::commands::status::StatusCommand;
use crate::commands::step::StepCommand;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub(crate) enum Command {
    Step(StepCommand),
    Status(StatusCommand),
}

impl Default for Command {
    fn default() -> Self {
        Self::Step(StepCommand {})
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}
