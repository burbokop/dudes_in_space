use crate::components::Components;
use crate::logger::StdOutLogger;
use clap::Parser;
use dudes_in_space_api::environment::Environment;

#[derive(Parser)]
pub(crate) struct StepCommand {}

impl StepCommand {
    pub(crate) fn exec(self, components: &Components, environment: &mut Environment) {
        environment.proceed(
            &components.process_token_context,
            &components.req_context,
            &components.objectives_decider_vault,
            &components.item_vault,
            &components.subordination_table,
            &mut StdOutLogger,
        );
    }
}
