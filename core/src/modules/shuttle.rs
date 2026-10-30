use crate::CORE_PACKAGE_ID;
use burbomath::Vector;
use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::{BankRegistry, WalletRegistry};
use dudes_in_space_api::item::{ItemSafe, ItemStorage, ItemVault, StorageRole};
use dudes_in_space_api::module::{
    AdminTradingConsole, CraftingConsole, DockError, DockingStatus, DockyardConsole,
    FlightControlConsole, FlyError, Module, ModuleCapability, ModuleConsole, ModuleId,
    ModuleStorage, ModuleTypeId, NavigationConsole, PackageId, TradingConsole, UndockError,
};
use dudes_in_space_api::person::{
    DynObjective, Logger, Person, PersonId, PersonSeed, StatusCollector,
};
use dudes_in_space_api::recipe::{
    AssemblyRecipe, InputItemRecipe, ItemRecipe, ModuleFactory, ModuleFactoryOutputDescription,
    OutputItemRecipe,
};
use dudes_in_space_api::utils::tagged_option::TaggedOptionSeed;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{
    DockingClamp, DockingConnector, ParentVesselInfo, VesselId, VesselModuleInterface,
};
use dyn_serde::{
    DynDeserializeSeed, DynDeserializeSeedVault, DynSerialize, TypeId, from_intermediate_seed,
};
use dyn_serde_macro::DeserializeSeedXXX;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_intermediate::{Intermediate, from_intermediate, to_intermediate};
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

pub(super) static TYPE_ID: &str = "Shuttle";
static FACTORY_TYPE_ID: &str = "ShuttleFactory";
static DOCKING_CONNECTOR_COMPAT_TYPE: usize = 0;
static DOCKING_DISTANCE: Float = 1.;
/// Max distance per iteration
static MAX_SPEED: Float = 10.;
static CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::FlightControl,
    ModuleCapability::Navigation,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
    ModuleCapability::PersonnelRoom,
    ModuleCapability::DockingConnector,
];
static PRIMARY_CAPABILITIES: &[ModuleCapability] = &[
    ModuleCapability::FlightControl,
    ModuleCapability::Engine,
    ModuleCapability::Reactor,
    ModuleCapability::FuelTank,
];

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::modules::shuttle::ShuttleSeed::<'v,'a,'b>)]
struct Shuttle {
    id: ModuleId,
    docking_connector: DockingConnector,
    #[serde(with = "dudes_in_space_api::utils::tagged_option")]
    #[deserialize_seed_xxx(seed = self.seed.person_seed)]
    pilot: Option<Person>,
}

#[derive(Clone)]
struct ShuttleSeed<'v, 'a, 'b> {
    person_seed: TaggedOptionSeed<PersonSeed<'v, 'a, 'b>>,
}

impl<'v, 'a, 'b> ShuttleSeed<'v, 'a, 'b> {
    fn new(
        vault: &'v DynDeserializeSeedVault<dyn DynObjective>,
        bank_registry: &'a BankRegistry,
        wallet_registry: &'b WalletRegistry,
    ) -> Self {
        ShuttleSeed {
            person_seed: TaggedOptionSeed::new(PersonSeed::new(
                vault,
                bank_registry,
                wallet_registry,
            )),
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

struct Console<'a, 'b> {
    id: &'a ModuleId,
    vessel_id: VesselId,
    parent_vessel_info: Option<&'b ParentVesselInfo>,
    this_vessel: &'b dyn VesselModuleInterface,
}

impl<'a, 'b> ModuleConsole for Console<'a, 'b> {
    fn id(&self) -> ModuleId {
        self.id.clone()
    }

    fn type_id(&self) -> ModuleTypeId {
        todo!()
    }

    fn package_id(&self) -> PackageId {
        todo!()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn interact(&mut self) -> bool {
        todo!()
    }

    fn in_progress(&self) -> bool {
        todo!()
    }

    fn crafting_console(&self) -> Option<&dyn CraftingConsole> {
        None
    }

    fn crafting_console_mut(&mut self) -> Option<&mut dyn CraftingConsole> {
        todo!()
    }

    fn dockyard_console(&self) -> Option<&dyn DockyardConsole> {
        todo!()
    }

    fn dockyard_console_mut(&mut self) -> Option<&mut dyn DockyardConsole> {
        todo!()
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        todo!()
    }

    fn trading_console_mut(&mut self) -> Option<&mut dyn TradingConsole> {
        todo!()
    }

    fn trading_admin_console(&self) -> Option<&dyn AdminTradingConsole> {
        todo!()
    }

    fn trading_admin_console_mut(&mut self) -> Option<&mut dyn AdminTradingConsole> {
        todo!()
    }

    fn storages(&self) -> Vec<&ItemStorage> {
        vec![]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        todo!()
    }

    fn storages_by_role_mut(&mut self, role: StorageRole) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn safes(&self) -> &[ItemSafe] {
        todo!()
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        todo!()
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        todo!()
    }

    fn docking_clamps_mut(&mut self) -> &mut [DockingClamp] {
        todo!()
    }

    fn flight_control_console(&self) -> Option<&dyn FlightControlConsole> {
        Some(self)
    }

    fn flight_control_console_mut(&mut self) -> Option<&mut dyn FlightControlConsole> {
        Some(self)
    }

    fn navigation_console(&self) -> Option<&dyn NavigationConsole> {
        Some(self)
    }

    fn navigation_console_mut(&mut self) -> Option<&mut dyn NavigationConsole> {
        Some(self)
    }
}

impl<'a, 'b> FlightControlConsole for Console<'a, 'b> {
    fn is_docked(&self) -> bool {
        self.parent_vessel_info.is_some()
    }

    fn can_undock(&self) -> bool {
        true
    }

    fn docking_status(
        &self,
        environment_context: &EnvironmentContext,
        vessel: &VesselId,
    ) -> DockingStatus {
        if let Some(parent_vessel_info) = self.parent_vessel_info
            && &parent_vessel_info.id == vessel
        {
            DockingStatus::Docked
        } else {
            let table = environment_context.vessel_spatial_table();
            let (Some(this), Some(target)) = (table.get(&self.vessel_id), table.get(vessel)) else {
                // TODO: Vessels created during this iteration are not in the table yet. Probably return some `Unknown` status
                return DockingStatus::TooFar;
            };
            if (target.pos - this.pos).len() > DOCKING_DISTANCE {
                DockingStatus::TooFar
            } else if target.free_clamps_count(DOCKING_CONNECTOR_COMPAT_TYPE) == 0 {
                DockingStatus::NoAvailableDockingPorts
            } else {
                DockingStatus::CanDock
            }
        }
    }

    fn dock(&self, vessel: &VesselId) -> Result<(), DockError> {
        self.this_vessel.request_dock(*vessel);
        Ok(())
    }

    fn undock(&self) -> Result<(), UndockError> {
        self.this_vessel.request_undock();
        Ok(())
    }

    fn fly(&self, heading: Vector<Float>) -> Result<(), FlyError> {
        let len = heading.len();
        let delta = if len > MAX_SPEED {
            heading * (MAX_SPEED / len)
        } else {
            heading
        };
        self.this_vessel.request_fly(delta);
        Ok(())
    }
}

impl<'a, 'b> NavigationConsole for Console<'a, 'b> {
    fn vector_to_vessel(
        &self,
        environment_context: &EnvironmentContext,
        vessel: &VesselId,
    ) -> Option<Vector<Float>> {
        let table = environment_context.vessel_spatial_table();
        Some(table.get(vessel)?.pos - table.get(&self.vessel_id)?.pos)
    }
}

impl Module for Shuttle {
    fn storages(&self) -> Vec<&ItemStorage> {
        vec![]
    }

    fn storages_mut(&mut self) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn module_storages(&self) -> &[ModuleStorage] {
        &[]
    }

    fn module_storages_mut(&mut self) -> &mut [ModuleStorage] {
        todo!()
    }

    fn proceed(
        &mut self,
        environment_context: &mut EnvironmentContext,
        this_vessel: &dyn VesselModuleInterface,
        logger: &mut dyn Logger,
    ) {
        let mut console = Console {
            id: &self.id,
            vessel_id: this_vessel.console().id(),
            parent_vessel_info: this_vessel.parent_vessel_info(),
            this_vessel,
        };

        if let Some(pilot) = &mut self.pilot {
            pilot.proceed(
                &mut rng(),
                environment_context,
                this_vessel.console(),
                &mut console,
                logger,
            )
        }
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
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
        CORE_PACKAGE_ID.into()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn docking_clamps(&self) -> &[DockingClamp] {
        &[]
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn trading_console(&self) -> Option<&dyn TradingConsole> {
        None
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
        &mut []
    }

    fn persons(&self) -> &[Person] {
        self.pilot.as_ref().map(std::slice::from_ref).unwrap_or(&[])
    }

    fn persons_mut(&mut self) -> &mut [Person] {
        self.pilot
            .as_mut()
            .map(std::slice::from_mut)
            .unwrap_or(&mut [])
    }

    fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_module(self);
        if let Some(pilot) = &self.pilot {
            pilot.collect_status(collector);
        }
        collector.exit_module();
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        &[]
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        &[]
    }

    fn safes(&self) -> &[ItemSafe] {
        &[]
    }

    fn safes_mut(&mut self) -> &mut [ItemSafe] {
        todo!()
    }

    fn storages_by_role(&self, role: StorageRole) -> Vec<&ItemStorage> {
        todo!()
    }

    fn storages_by_role_mut(&mut self, role: StorageRole) -> Vec<&mut ItemStorage> {
        todo!()
    }

    fn active(&self) -> bool {
        todo!()
    }
}

pub(crate) struct ShuttleDynSeed {
    vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
    bank_registry: Rc<BankRegistry>,
    wallet_registry: Rc<WalletRegistry>,
}

impl ShuttleDynSeed {
    pub(crate) fn new(
        vault: Rc<DynDeserializeSeedVault<dyn DynObjective>>,
        bank_registry: Rc<BankRegistry>,
        wallet_registry: Rc<WalletRegistry>,
    ) -> Self {
        Self {
            vault,
            bank_registry,
            wallet_registry,
        }
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
        let obj: Shuttle = from_intermediate_seed(
            ShuttleSeed::new(&self.vault, &self.bank_registry, &self.wallet_registry),
            &intermediate,
        )
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
    fn create(&self, item_vault: Rc<ItemVault>, _: &InputItemRecipe) -> Box<dyn Module> {
        Box::new(Shuttle {
            id: ModuleId::new_v4(),
            docking_connector: DockingConnector::new(DOCKING_CONNECTOR_COMPAT_TYPE),
            pilot: None,
        })
    }

    fn output_description(&self) -> &dyn ModuleFactoryOutputDescription {
        self
    }
}

impl ModuleFactoryOutputDescription for ShuttleFactory {
    fn type_id(&self) -> ModuleTypeId {
        TYPE_ID.into()
    }

    fn capabilities(&self) -> &[ModuleCapability] {
        CAPABILITIES
    }

    fn primary_capabilities(&self) -> &[ModuleCapability] {
        PRIMARY_CAPABILITIES
    }

    fn item_recipes(&self) -> &[ItemRecipe] {
        &[]
    }

    fn output_item_recipes(&self) -> &[OutputItemRecipe] {
        &[]
    }

    fn input_item_recipes(&self) -> &[InputItemRecipe] {
        &[]
    }

    fn assembly_recipes(&self) -> &[AssemblyRecipe] {
        &[]
    }
}
