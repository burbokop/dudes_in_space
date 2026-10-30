use crate::environment::{RequestStorage, VesselSpatialTable};
use crate::finance::{BankRegistry, CurrencyGenerator, WalletRegistry};
use crate::item::ItemVault;
use crate::module::ProcessTokenContext;
use crate::person::{ObjectiveDeciderVault, SubordinationTable};
use crate::trade::OrderHolder;
use std::rc::Rc;

pub struct EnvironmentContext<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h> {
    process_token_context: &'a ProcessTokenContext,
    request_storage: &'b mut RequestStorage,
    objective_decider_vault: &'c ObjectiveDeciderVault,
    subordination_table: &'d SubordinationTable,
    bank_registry: &'e BankRegistry,
    wallet_registry: &'f WalletRegistry,
    currency_generator: &'g CurrencyGenerator,
    order_holder: &'h OrderHolder,
    item_vault: Rc<ItemVault>,
    vessel_spatial_table: VesselSpatialTable,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h> EnvironmentContext<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h> {
    pub fn new(
        process_token_context: &'a ProcessTokenContext,
        request_storage: &'b mut RequestStorage,
        objective_decider_vault: &'c ObjectiveDeciderVault,
        subordination_table: &'d SubordinationTable,
        bank_registry: &'e BankRegistry,
        wallet_registry: &'f WalletRegistry,
        currency_generator: &'g CurrencyGenerator,
        order_holder: &'h OrderHolder,
        item_vault: Rc<ItemVault>,
        vessel_spatial_table: VesselSpatialTable,
    ) -> Self {
        Self {
            process_token_context,
            request_storage,
            objective_decider_vault,
            subordination_table,
            bank_registry,
            wallet_registry,
            currency_generator,
            order_holder,
            item_vault,
            vessel_spatial_table,
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

    pub fn objective_decider_vault(&self) -> &ObjectiveDeciderVault {
        &self.objective_decider_vault
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

    pub fn order_holder(&self) -> &OrderHolder {
        self.order_holder
    }

    pub fn item_vault(&self) -> &Rc<ItemVault> {
        &self.item_vault
    }

    pub fn vessel_spatial_table(&self) -> &VesselSpatialTable {
        &self.vessel_spatial_table
    }
}
