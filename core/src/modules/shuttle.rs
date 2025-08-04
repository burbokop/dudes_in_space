use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::module::{DefaultModuleConsole, Module, ModuleCapability, ModuleId, ModuleStorage, ModuleTypeId, PackageId,  TradingConsole};
use dudes_in_space_api::person::{
    DynObjective, Logger, ObjectiveDeciderVault, Person, PersonId, PersonSeed,
};
use dudes_in_space_api::recipe::{AssemblyRecipe, InputRecipe, ModuleFactory, Recipe};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::vessel::{DockingClamp, DockingConnector, VesselModuleInterface};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;
use rand::rng;
use dudes_in_space_api::environment::EnvironmentContext;

static TYPE_ID: &str = "Shuttle";
static FACTORY_TYPE_ID: &str = "ShuttleFactory";
static DOCKING_CONNECTOR_COMPAT_TYPE: usize = 0;
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
    ModuleCapability::PersonnelRoom,
    ModuleCapability::DockingConnector,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::Cockpit,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::shuttle::ShuttleSeed::<'v>)]
struct Shuttle {
    id: ModuleId,
    docking_connector: DockingConnector,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    pilot: Option<Person>,
}

#[derive(Clone)]
struct ShuttleSeed<'v> {
    person_seed: TaggedOptionSeed<PersonSeed<'v>>,
}

impl<'v> ShuttleSeed<'v> {
    fn new(vault: &'v DynDeserializeSeedVault<dyn DynObjective>) -> Self {
        ShuttleSeed {
            person_seed: TaggedOptionSeed::new(PersonSeed::new(vault)),
        }
    }
}

impl DynSerialize for Shuttle {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

impl Module for Shuttle {
    fn storages(&self) -> &[ItemStorage] {
        todo!()
    }

    fn storages_mut(&mut self) -> &mut [ItemStorage] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        todo!()
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn proceed(
        &mut self,
        this_vessel: &dyn VesselModuleInterface,
        environment_context: &mut EnvironmentContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        let mut console = DefaultModuleConsole::new(
            self.id,
            CAPABILITIES,
            PRIMARY_CAPABILITIES,
        );

        if let Some(pilot) = &mut self.pilot {
            pilot.proceed(
                &mut rng(),
                &mut console,
                this_vessel.console(),
                environment_context,
                decider_vault,
                logger,
            )
        }
    }

    fn recipes(&self) -> Vec<Recipe> {
        todo!()
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        todo!()
    }

    fn extract_person(&mut self, id: PersonId) -> Option<Person> {
        todo!()
    }

    fn insert_person(&mut self, person: Person) -> bool {
        match &self.pilot {
            None => {
                self.pilot = Some(person);
                true
            }
            Some(_) => false,
        }
    }

    fn contains_person(&self, id: PersonId) -> bool {
        self.pilot.as_ref().map(|p| p.id() == id).unwrap_or(false)
    }

    fn id(&self) -> ModuleId {
        self.id
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }

    fn free_person_slots_count(&self) -> usize {
        const CAPACITY: usize = 1;
        CAPACITY - self.pilot.iter().len()
    }

    fn docking_connectors(&self) -> &[DockingConnector] {
        std::slice::from_ref(&self.docking_connector)
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }
}

pub(crate) struct ShuttleDynSeed {
    vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
}

impl ShuttleDynSeed {
    pub(crate) fn new(vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>) -> Self {
        Self { vault }
    }
}

impl DynDeserializeSeed<dyn Module> for ShuttleDynSeed {
    fn type_id(&self) -> TypeId {
        TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        _: &DynDeserializeSeedVault<dyn Module>,
    ) -> Result<Box<dyn Module>, Box<dyn Error>> {
        let obj: Shuttle = from_intermediate_seed(ShuttleSeed::new(&self.vault), &intermediate)
            .map_err(|e| e.to_string())?;
        Ok(Box::new(obj))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ShuttleFactory {}

impl DynSerialize for ShuttleFactory {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn serialize(&self) -> Result<Intermediate, Box<dyn Error>> {
        to_intermediate(self).map_err(|e| e.into())
    }
}

pub(crate) struct ShuttleFactoryDynSeed;

impl DynDeserializeSeed<dyn ModuleFactory> for ShuttleFactoryDynSeed {
    fn type_id(&self) -> TypeId {
        FACTORY_TYPE_ID.to_string()
    }

    fn deserialize(
        &self,
        intermediate: Intermediate,
        this_vault: &DynDeserializeSeedVault<dyn ModuleFactory>,
    ) -> Result<Box<dyn ModuleFactory>, Box<dyn Error>> {
        let r: Box<ShuttleFactory> = from_intermediate(&intermediate).map_err(|e| e.to_string())?;
        Ok(r)
    }
}

impl ModuleFactory for ShuttleFactory {
    fn output_type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn create(&self, _: &InputRecipe) -> Box<dyn Module> {
        Box::new(Shuttle {
            id: ModuleId::new_v4(),
            docking_connector: DockingConnector::new(DOCKING_CONNECTOR_COMPAT_TYPE),
            pilot: None,
        })
    }

    fn output_capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn output_primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }
}
