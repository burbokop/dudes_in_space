use crate::environment::RequestStorage;
use crate::finance::{BankRegistry, CurrencyGenerator};
use crate::module::ProcessTokenContext;
use crate::person::SubordinationTable;

pub struct EnvironmentContext<'a, 'b, 'c, 'd, 'e> {
    process_token_context: &'a ProcessTokenContext,
    request_storage: &'b mut RequestStorage,
    subordination_table: &'c SubordinationTable,
    bank_registry: &'d BankRegistry,
    currency_generator: &'e CurrencyGenerator,
}

impl<'a, 'b, 'c, 'd, 'e> EnvironmentContext<'a, 'b, 'c, 'd, 'e> {
    pub fn new(
        process_token_context: &'a ProcessTokenContext,
        request_storage: &'b mut RequestStorage,
        subordination_table: &'c SubordinationTable,
        bank_registry: &'d BankRegistry,
        currency_generator: &'e CurrencyGenerator,
    ) -> Self {
        Self {
            process_token_context,
            request_storage,
            subordination_table,
            bank_registry,
            currency_generator,
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

    pub fn bank_registry(&self) -> &BankRegistry {
        self.bank_registry
    }

    pub fn currency_generator(&self) -> &CurrencyGenerator {
        self.currency_generator
    }
}
