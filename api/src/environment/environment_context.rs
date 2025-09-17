use crate::environment::RequestStorage;
use crate::finance::{BankRegistry, CurrencyGenerator, WalletRegistry};
use crate::item::ItemVault;
use crate::module::ProcessTokenContext;
use crate::person::SubordinationTable;

pub struct EnvironmentContext<'a, 'b, 'c, 'd, 'e, 'f, 'g> {
    process_token_context: &'a ProcessTokenContext,
    request_storage: &'b mut RequestStorage,
    subordination_table: &'c SubordinationTable,
    bank_registry: &'d BankRegistry,
    wallet_registry: &'e WalletRegistry,
    currency_generator: &'f CurrencyGenerator,
    item_vault: &'g ItemVault,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g> EnvironmentContext<'a, 'b, 'c, 'd, 'e, 'f, 'g> {
    pub fn new(
        process_token_context: &'a ProcessTokenContext,
        request_storage: &'b mut RequestStorage,
        subordination_table: &'c SubordinationTable,
        bank_registry: &'d BankRegistry,
        wallet_registry: &'e WalletRegistry,
        currency_generator: &'f CurrencyGenerator,
        item_vault: &'g ItemVault,
    ) -> Self {
        Self {
            process_token_context,
            request_storage,
            subordination_table,
            bank_registry,
            wallet_registry,
            currency_generator,
            item_vault,
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

    pub fn wallet_registry(&self) -> &WalletRegistry {
        self.wallet_registry
    }

    pub fn currency_generator(&self) -> &CurrencyGenerator {
        self.currency_generator
    }

    pub fn item_vault(&self) -> &ItemVault {
        self.item_vault
    }
}
