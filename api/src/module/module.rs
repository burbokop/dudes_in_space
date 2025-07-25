use crate::item::ItemStorage;
use crate::module::{ModuleCapability, ModuleStorage, ProcessTokenContext};
use crate::person::{Person, PersonId};
use crate::recipe::{AssemblyRecipe, Recipe};
use crate::vessel::VesselModuleInterface;
use dyn_serde::DynSerialize;
use dyn_serde_macro::dyn_serde_trait;
use std::fmt::Debug;
use uuid::Uuid;

pub type PackageId = String;
pub type ModuleTypeId = String;
pub type ModuleId = Uuid;

pub trait Module: Debug + DynSerialize {
    /// common
    fn id(&self) -> ModuleId;
    fn package_id(&self) -> PackageId;
    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        process_token_context: &ProcessTokenContext,
    );
    fn capabilities(&self) -> &[ModuleCapability];

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
    fn storages(&mut self) -> &mut [ItemStorage];
    fn module_storages(&mut self) -> &mut [ModuleStorage];
}

dyn_serde_trait!(Module);
