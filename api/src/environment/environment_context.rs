use crate::environment::RequestStorage;
use crate::module::ProcessTokenContext;
use crate::person::SubordinationTable;

pub struct EnvironmentContext<'ptc, 'rs, 'st> {
    process_token_context: &'ptc ProcessTokenContext,
    request_storage: &'rs mut RequestStorage,
    subordination_table: &'st SubordinationTable,
}

impl<'ptc, 'rs, 'st> EnvironmentContext<'ptc, 'rs, 'st> {
    pub fn new(
        process_token_context: &'ptc ProcessTokenContext,
        request_storage: &'rs mut RequestStorage,
        subordination_table: &'st SubordinationTable,
    ) -> Self {
        Self {
            process_token_context,
            request_storage,
            subordination_table,
        }
    }

    pub fn process_token_context(&self) -> &ProcessTokenContext {
        &self.process_token_context
    }

    pub fn request_storage(&self) -> &RequestStorage {
        &self.request_storage
    }

    pub fn request_storage_mut(&mut self) -> &mut RequestStorage {
        &mut self.request_storage
    }

    pub fn subordination_table(&self) -> &SubordinationTable {
        &self.subordination_table
    }
}
