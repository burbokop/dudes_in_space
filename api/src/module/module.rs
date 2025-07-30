use crate::item::ItemStorage;
use crate::module::{ModuleCapability, ModuleStorage, ProcessTokenContext, TradingConsole};
use crate::person::{Logger, ObjectiveDeciderVault, Person, PersonId};
use crate::recipe::{AssemblyRecipe, Recipe};
use crate::vessel::{DockingClamp, VesselModuleInterface};
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use std::fmt::Debug;
use uuid::{NonNilUuid, Uuid};

pub type PackageId = String;
pub type ModuleTypeId = String;
pub type ModuleId = NonNilUuid;

pub trait Module: Debug + DynSerialize {
    /// common
    fn id(&self) -> ModuleId;
    fn package_id(&self) -> PackageId;
    fn capabilities(&self) -> &[ModuleCapability];
    fn primary_capabilities(&self) -> &[ModuleCapability];

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    );

    /// crafting
    fn recipes(&self) -> Vec<Recipe>;
    /// assembly
    fn assembly_recipes(&self) -> &[AssemblyRecipe];

    /// persons
    fn extract_person(&mut self, id: PersonId) -> Option<Person>;
    fn insert_person(&mut self, person: Person) -> bool;
    fn can_insert_person(&self) -> bool;
    fn contains_person(&self, id: PersonId) -> bool;

    /// storage
    fn storages(&self) -> &[ItemStorage];
    fn storages_mut(&mut self) -> &mut [ItemStorage];

    fn module_storages(&self) -> &[ModuleStorage];
    fn module_storages_mut(&mut self) -> &mut [ModuleStorage];

    fn docking_clamps(&self) -> &[DockingClamp];

    fn trading_console(&self) -> Option<&dyn TradingConsole>;
    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole>;
}

dyn_serde_trait!(Module, ModuleSeed);
