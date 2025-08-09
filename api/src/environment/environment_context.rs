use crate::environment::RequestStorage;
use crate::module::ProcessTokenContext;

pub struct EnvironmentContext<'ptc, 'rs> {
    process_token_context: &'ptc ProcessTokenContext,
    request_storage: &'rs mut RequestStorage,
}

impl<'ptc, 'rs> EnvironmentContext<'ptc, 'rs> {
    pub fn new(
        process_token_context: &'ptc ProcessTokenContext,
        request_storage: &'rs mut RequestStorage,
    ) -> Self {
        Self {
            process_token_context,
            request_storage,
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
}
