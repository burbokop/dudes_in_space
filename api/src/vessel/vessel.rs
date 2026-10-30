use crate::environment::{DockingRequest, EnvironmentContext};
use crate::module::{Module, ModuleCapability, ModuleConsole, ModuleId, ModuleSeed};
use crate::person;
use crate::person::{
    Logger, PersonId, StatusCollector, SubordinationTable, VesselPermission, VesselPermissions,
};
use crate::utils::non_nil_uuid::NonNilUuid;
use crate::utils::utils::Float;
use crate::vessel::{
    DockingConnectorId, MoveToDockedVesselError, MoveToModuleError, VesselConsole,
    VesselInternalConsole, VesselModuleInterface,
};
use burbomath::{Point, Vector};
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeSet;
use std::fmt::Formatter;
use std::iter;
use std::ops::{ControlFlow, Deref, DerefMut, Try};

pub type VesselId = NonNilUuid;

pub struct VesselIdPathRef<'a>(&'a [VesselId]);

impl<'a> VesselIdPathRef<'a> {
    pub fn leaf(&self) -> &'a VesselId {
        self.0.last().unwrap()
    }

    pub fn parent(&self) -> Option<VesselId> {
        let l = self.0.len();
        if l > 1 { Some(self.0[l - 2]) } else { None }
    }

    pub fn to_owned(&self) -> VesselIdPath {
        VesselIdPath(self.0.to_vec())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VesselIdPath(Vec<VesselId>);

impl VesselIdPath {
    pub fn leaf(&self) -> &VesselId {
        self.0.last().unwrap()
    }

    pub fn as_ref<'a>(&'a self) -> VesselIdPathRef<'a> {
        VesselIdPathRef(&self.0)
    }

    pub fn is_ancestor(&self, id: &VesselId) -> bool {
        self.0.contains(id)
    }

    pub fn is_parent(&self, id: &VesselId) -> bool {
        let l = self.0.len();
        if l > 1 { (&self.0[l - 2]) == id } else { false }
    }
}

pub struct EmptyVesselIdPathError;

impl<'a> TryFrom<&'a [VesselId]> for VesselIdPathRef<'a> {
    type Error = EmptyVesselIdPathError;

    fn try_from(value: &'a [VesselId]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<Vec<VesselId>> for VesselIdPath {
    type Error = EmptyVesselIdPathError;

    fn try_from(value: Vec<VesselId>) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
enum VesselRequest {
    MoveToModule {
        person_id: PersonId,
        module_id: ModuleId,
    },
    MoveToDockedVessel {
        person_id: PersonId,
        target_module_id: ModuleId,
        connector_id: DockingConnectorId,
    },
    AddModule {
        module: Box<dyn Module>,
    },
    Undock,
    Dock {
        target: VesselId,
    },
    Fly {
        delta: Vector<Float>,
    },
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::VesselSeed::<'v>)]
pub struct Vessel {
    id: VesselId,
    name: String,
    owner: PersonId,
    pos: Point<Float>,
    #[deserialize_seed_xxx(seed = self.seed.module_seq_seed)]
    modules: Vec<RefCell<Box<dyn Module>>>,
    #[serde(skip)]
    requests: RefCell<Vec<VesselRequest>>,
    #[serde(default)]
    permissions: VesselPermissions,
}

pub struct ParentVesselInfo {
    pub id: VesselId,
}

#[derive(Clone)]
struct ModuleSeqSeed<'v> {
    module_seed: ModuleSeed<'v>,
}

impl<'de, 'v> DeserializeSeed<'de> for ModuleSeqSeed<'v> {
    type Value = Vec<RefCell<Box<dyn Module>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ModuleSeqVisitor<'v> {
            module_seed: ModuleSeed<'v>,
        }

        impl<'b, 'de> Visitor<'de> for ModuleSeqVisitor<'b> {
            type Value = Vec<RefCell<Box<dyn Module>>>;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("list")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut modules: Vec<RefCell<Box<dyn Module>>> = Default::default();
                while let Some(key) = seq.next_element_seed(self.module_seed.clone())? {
                    modules.push(RefCell::new(key));
                }
                Ok(modules)
            }
        }

        deserializer.deserialize_seq(ModuleSeqVisitor {
            module_seed: self.module_seed,
        })
    }
}

#[derive(Clone)]
pub(crate) struct VesselSeed<'v> {
    module_seq_seed: ModuleSeqSeed<'v>,
}

impl<'v> VesselSeed<'v> {
    pub(crate) fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> Self {
        Self {
            module_seq_seed: ModuleSeqSeed {
                module_seed: ModuleSeed::new(vault),
            },
        }
    }
}

impl Vessel {
    pub fn id(&self) -> VesselId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn owner(&self) -> PersonId {
        self.owner
    }

    pub fn pos(&self) -> Point<Float> {
        self.pos
    }

    pub(crate) fn set_pos(&mut self, pos: Point<Float>) {
        self.pos = pos
    }

    /// Recursively find a clamp holding vessel with `id` and undock it
    pub(crate) fn undock_vessel(&self, id: VesselId) -> Option<Vessel> {
        for module in &self.modules {
            let mut module = module.borrow_mut();
            for clamp in module.docking_clamps_mut() {
                let Some(connection) = clamp.connection() else {
                    continue;
                };
                if connection.vessel.id == id {
                    return clamp.undock();
                }
                if let Some(vessel) = connection.vessel.undock_vessel(id) {
                    return Some(vessel);
                }
            }
        }
        None
    }

    /// Recursively find vessel with `target` id and dock `vessel` to its free compatible clamp
    pub(crate) fn dock_vessel(&self, target: VesselId, mut vessel: Vessel) -> Result<(), Vessel> {
        if self.id == target {
            let connectors: Vec<_> = vessel
                .modules()
                .flat_map(|module| {
                    module
                        .docking_connectors()
                        .iter()
                        .map(|connector| (connector.id(), connector.compat_type()))
                        .collect::<Vec<_>>()
                })
                .collect();
            for module in &self.modules {
                let mut module = module.borrow_mut();
                for clamp in module.docking_clamps_mut() {
                    if !clamp.is_empty() {
                        continue;
                    }
                    if let Some((connector_id, _)) = connectors
                        .iter()
                        .find(|(_, compat_type)| *compat_type == clamp.compat_type())
                    {
                        clamp.dock_by_connector_id(vessel, *connector_id).unwrap();
                        return Ok(());
                    }
                }
            }
            return Err(vessel);
        }
        for module in &self.modules {
            for clamp in module.borrow().docking_clamps() {
                if let Some(connection) = clamp.connection() {
                    vessel = match connection.vessel.dock_vessel(target, vessel) {
                        Ok(()) => return Ok(()),
                        Err(vessel) => vessel,
                    };
                }
            }
        }
        Err(vessel)
    }

    pub fn traverse<R>(&self, mut f: impl FnMut(VesselIdPathRef, &Vessel) -> R) -> R
    where
        R: Try<Output = ()>,
    {
        self.traverse_impl(vec![], &mut f)
    }

    fn traverse_impl<R>(
        &self,
        mut path: Vec<VesselId>,
        f: &mut impl FnMut(VesselIdPathRef, &Vessel) -> R,
    ) -> R
    where
        R: Try<Output = ()>,
    {
        path.push(self.id);
        match f(VesselIdPathRef(&path), self).branch() {
            ControlFlow::Continue(_) => {}
            ControlFlow::Break(g) => return R::from_residual(g),
        }
        for module in &self.modules {
            let module = module.borrow();
            for clamp in module.docking_clamps() {
                if let Some(connection) = clamp.connection() {
                    match connection
                        .vessel
                        .traverse_impl::<R>(path.clone(), f)
                        .branch()
                    {
                        ControlFlow::Continue(c) => {}
                        ControlFlow::Break(b) => return R::from_residual(b),
                    }
                }
            }
        }
        R::from_output(())
    }

    pub(crate) fn has_empty_pilot_seat(&self) -> bool {
        self.modules_with_capability(ModuleCapability::FlightControl)
            .any(|cockpit| cockpit.free_person_slots_count() > 0)
    }

    pub fn new(
        name: String,
        owner: PersonId,
        pos: Point<Float>,
        modules: Vec<Box<dyn Module>>,
    ) -> Self {
        Self {
            id: VesselId::new_v4(),
            name,
            owner,
            pos,
            modules: modules.into_iter().map(RefCell::new).collect(),
            requests: Default::default(),
            permissions: Default::default(),
        }
    }

    pub fn modules<'a>(&'a self) -> impl Iterator<Item = Ref<'a, dyn Module>> {
        self.modules
            .iter()
            .map(|module| Ref::map(module.borrow(), |x| x.deref()))
    }

    pub fn modules_mut<'a>(&'a self) -> impl Iterator<Item = RefMut<'a, dyn Module>> {
        self.modules.iter().map(|module| {
            RefMut::map(module.borrow_mut(), |x| {
                let x = x.deref_mut();
                x
            })
        })
    }

    pub fn modules_with_capability<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> impl Iterator<Item = Ref<'a, dyn Module>> {
        self.modules.iter().filter_map(move |module| {
            if let Ok(module) = module.try_borrow() {
                if module.capabilities().contains(&cap) {
                    return Some(Ref::map(module, |x| x.deref()));
                }
            }
            None
        })
    }

    pub fn modules_with_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> impl Iterator<Item = RefMut<'a, dyn Module>> {
        self.modules.iter().filter_map(move |module| {
            if let Ok(module) = module.try_borrow_mut() {
                if module.capabilities().contains(&cap) {
                    return Some(RefMut::map(module, |x| {
                        let x = x.deref_mut();
                        x
                    }));
                }
            }
            None
        })
    }

    pub fn modules_with_primary_capability<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> impl Iterator<Item = Ref<'a, dyn Module>> {
        self.modules.iter().filter_map(move |module| {
            if let Ok(module) = module.try_borrow() {
                if module.primary_capabilities().contains(&cap) {
                    return Some(Ref::map(module, |x| x.deref()));
                }
            }
            None
        })
    }

    pub fn modules_with_primary_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> impl Iterator<Item = RefMut<'a, dyn Module>> {
        #[allow(unreachable_code)]
        iter::once(todo!())
    }

    pub(crate) fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(RefCell::new(module));
    }

    pub(crate) fn proceed(
        &mut self,
        environment_context: &mut EnvironmentContext,
        parent_vessel_info: Option<&ParentVesselInfo>,
        logger: &mut dyn Logger,
    ) {
        for v in &self.modules {
            v.borrow_mut().proceed(
                environment_context,
                &VesselModuleInterfaceImpl {
                    vessel: self,
                    parent_vessel_info,
                },
                logger,
            )
        }
        for request in self.requests.take() {
            match request {
                VesselRequest::MoveToModule {
                    person_id,
                    module_id,
                } => {
                    let src = self
                        .modules
                        .iter()
                        .find(|m| m.borrow().contains_person(person_id))
                        .unwrap();
                    let dst = self
                        .modules
                        .iter()
                        .find(|m| m.borrow().id() == module_id)
                        .unwrap();
                    assert_ne!(src.as_ptr(), dst.as_ptr());
                    let mut src = src.borrow_mut();
                    let mut dst = dst.borrow_mut();
                    if dst.free_person_slots_count() == 0 {
                        panic!(
                            "Can not insert person to module `{}` of type `{}`",
                            dst.id(),
                            dst.type_id()
                        )
                    }
                    let ok = dst.insert_person(src.extract_person(person_id).unwrap());
                    assert!(ok);
                    assert!(!src.contains_person(person_id));
                    assert!(dst.contains_person(person_id));
                }
                VesselRequest::MoveToDockedVessel {
                    person_id,
                    target_module_id,
                    connector_id,
                } => {
                    let clamps: Vec<_> = self
                        .modules_with_capability_mut(ModuleCapability::DockingClamp)
                        .collect();

                    let mut clamp_module = clamps
                        .into_iter()
                        .find(|clamp_module| {
                            person::utils::find_docking_clamp_with_connector_with_id(
                                clamp_module.docking_clamps(),
                                connector_id,
                            )
                            .is_some()
                        })
                        .unwrap();

                    let person = clamp_module.extract_person(person_id).unwrap();

                    let clamp = person::utils::find_docking_clamp_with_connector_with_id(
                        clamp_module.docking_clamps(),
                        connector_id,
                    )
                    .unwrap();

                    let mut connector_module = clamp
                        .connection()
                        .unwrap()
                        .vessel
                        .modules_with_capability_mut(ModuleCapability::DockingConnector)
                        .find(|x| {
                            x.docking_connectors()
                                .iter()
                                .find(|c| c.id() == connector_id)
                                .is_some()
                        })
                        .unwrap();

                    if connector_module.free_person_slots_count() == 0 {
                        panic!(
                            "Can not insert person to module `{}` of type `{}`",
                            connector_module.id(),
                            connector_module.type_id()
                        )
                    }
                    let ok = connector_module.insert_person(person);
                    assert!(ok);
                    assert!(connector_module.contains_person(person_id));
                    assert!(!clamp_module.contains_person(person_id));
                }
                VesselRequest::AddModule { module } => {
                    self.modules.push(RefCell::new(module));
                }
                VesselRequest::Undock => {
                    if parent_vessel_info.is_some() {
                        environment_context
                            .push_docking_request(DockingRequest::Undock { vessel: self.id });
                    }
                }
                VesselRequest::Dock { target } => {
                    if parent_vessel_info.is_none() {
                        environment_context.push_docking_request(DockingRequest::Dock {
                            vessel: self.id,
                            target,
                        });
                    }
                }
                VesselRequest::Fly { delta } => {
                    if parent_vessel_info.is_none() {
                        self.pos = self.pos + delta;
                    }
                }
            }
        }
    }

    pub fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_vessel(self);
        for module in &self.modules {
            module.borrow().collect_status(collector);
        }
        collector.exit_vessel();
    }
}

struct VesselModuleInterfaceImpl<'a, 'b> {
    vessel: &'a Vessel,
    parent_vessel_info: Option<&'b ParentVesselInfo>,
}

impl<'a, 'b> VesselModuleInterface for VesselModuleInterfaceImpl<'a, 'b> {
    fn add_module(&self, module: Box<dyn Module>) {
        self.vessel
            .requests
            .borrow_mut()
            .push(VesselRequest::AddModule { module })
    }

    fn console(&self) -> &dyn VesselInternalConsole {
        self.vessel
    }

    fn parent_vessel_info(&self) -> Option<&ParentVesselInfo> {
        self.parent_vessel_info
    }

    fn request_undock(&self) {
        self.vessel
            .requests
            .borrow_mut()
            .push(VesselRequest::Undock)
    }

    fn request_dock(&self, target: VesselId) {
        self.vessel
            .requests
            .borrow_mut()
            .push(VesselRequest::Dock { target })
    }

    fn request_fly(&self, delta: Vector<Float>) {
        self.vessel
            .requests
            .borrow_mut()
            .push(VesselRequest::Fly { delta })
    }
}

impl VesselConsole for Vessel {
    fn id(&self) -> VesselId {
        self.id
    }

    fn owner(&self) -> PersonId {
        self.owner
    }

    fn capabilities(&self) -> BTreeSet<ModuleCapability> {
        self.modules
            .iter()
            .filter_map(|module| {
                if let Ok(module) = module.try_borrow() {
                    Some(
                        module
                            .capabilities()
                            .into_iter()
                            .cloned()
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    fn primary_capabilities(&self) -> BTreeSet<ModuleCapability> {
        self.modules
            .iter()
            .filter_map(|module| {
                if let Ok(module) = module.try_borrow() {
                    Some(
                        module
                            .primary_capabilities()
                            .into_iter()
                            .cloned()
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    fn permissions(&self) -> &VesselPermissions {
        &self.permissions
    }
}

impl VesselInternalConsole for Vessel {
    fn modules_with_capability<'a>(&'a self, cap: ModuleCapability) -> Vec<Ref<'a, dyn Module>> {
        self.modules_with_capability(cap).collect()
    }

    fn modules_with_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> Vec<RefMut<'a, dyn Module>> {
        self.modules_with_capability_mut(cap).collect()
    }

    fn modules_with_primary_capability<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> Vec<Ref<'a, dyn Module>> {
        self.modules_with_primary_capability(cap).collect()
    }

    fn modules_with_primary_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> Vec<RefMut<'a, dyn Module>> {
        self.modules_with_primary_capability_mut(cap).collect()
    }

    fn move_person_to_module(
        &self,
        subordination_table: &SubordinationTable,
        person_id: PersonId,
        module_id: ModuleId,
    ) -> Result<(), MoveToModuleError> {
        let pending_requests = self
            .requests
            .borrow()
            .iter()
            .filter(|r| {
                let m = &module_id;
                match r {
                    VesselRequest::MoveToModule { module_id, .. } => module_id == m,
                    VesselRequest::MoveToDockedVessel { connector_id, .. } => {
                        todo!()
                    }
                    VesselRequest::AddModule { .. }
                    | VesselRequest::Undock
                    | VesselRequest::Dock { .. }
                    | VesselRequest::Fly { .. } => false,
                }
            })
            .count();

        let module = self
            .module_by_id(module_id)
            .ok_or(MoveToModuleError::ModuleNotFound)?;

        if let Some(permissions_needed) = if module
            .capabilities()
            .contains(&ModuleCapability::FlightControl)
        {
            Some(VesselPermission::Pilot)
        } else if module.capabilities().contains(&ModuleCapability::Dockyard)
            || module
                .capabilities()
                .contains(&ModuleCapability::ModuleCrafting)
            || module
                .capabilities()
                .contains(&ModuleCapability::ItemCrafting)
            || module
                .capabilities()
                .contains(&ModuleCapability::TradingTerminal)
            || module
                .capabilities()
                .contains(&ModuleCapability::VesselSellingTerminal)
        {
            Some(VesselPermission::Operate)
        } else {
            None
        } {
            if !subordination_table.has_permission(person_id, self, permissions_needed) {
                return Err(MoveToModuleError::PermissionDenied);
            }
        }

        if module.free_person_slots_count() < pending_requests + 1 {
            return Err(MoveToModuleError::NotEnoughSpace);
        }

        self.requests
            .borrow_mut()
            .push(VesselRequest::MoveToModule {
                person_id,
                module_id,
            });

        Ok(())
    }

    fn move_person_to_docked_vessel(
        &self,
        subordination_table: &SubordinationTable,
        this_module: &dyn ModuleConsole,
        person_id: PersonId,
        connector_id: DockingConnectorId,
    ) -> Result<(), MoveToDockedVesselError> {
        let clamp_modules: Vec<_> = self
            .modules_with_capability(ModuleCapability::DockingClamp)
            .collect();

        let mut clamps = clamp_modules
            .iter()
            .map(|clamp_module| clamp_module.docking_clamps())
            .flatten()
            .chain(this_module.docking_clamps());

        let (target_module, target_vessel) = clamps
            .find_map(|clamp| {
                clamp.connection().and_then(|connection| {
                    connection
                        .vessel
                        .modules
                        .iter()
                        .find(|connector_module| {
                            connector_module
                                .borrow()
                                .docking_connectors()
                                .iter()
                                .find(|other_vessel_module_connector| {
                                    other_vessel_module_connector.id() == connector_id
                                })
                                .is_some()
                        })
                        .map(|module| (module, &connection.vessel))
                })
            })
            .unwrap();

        let target_module = target_module.borrow();
        let target_module_id = target_module.id();

        let permissions_needed = if target_module
            .capabilities()
            .contains(&ModuleCapability::FlightControl)
        {
            VesselPermission::Pilot
        } else {
            VesselPermission::Enter
        };

        if !subordination_table.has_permission(person_id, target_vessel, permissions_needed) {
            todo!()
        }

        let pending_requests = self
            .requests
            .borrow()
            .iter()
            .filter(|r| {
                let c = &connector_id;
                let m = &target_module_id;
                match r {
                    VesselRequest::MoveToDockedVessel {
                        target_module_id, ..
                    } => target_module_id == m,
                    _ => false,
                }
            })
            .count()
            + target_vessel
                .requests
                .borrow()
                .iter()
                .filter(|r| {
                    let c = &connector_id;
                    let m = &target_module_id;
                    match r {
                        VesselRequest::MoveToModule { module_id, .. } => module_id == m,
                        _ => false,
                    }
                })
                .count();

        if target_module.free_person_slots_count() < pending_requests + 1 {
            return Err(MoveToDockedVesselError::NotEnoughSpace);
        }

        self.requests
            .borrow_mut()
            .push(VesselRequest::MoveToDockedVessel {
                person_id,
                target_module_id,
                connector_id,
            });

        Ok(())
    }

    fn module_by_id<'a>(&'a self, id: ModuleId) -> Option<Ref<'a, dyn Module>> {
        self.modules
            .iter()
            .find_map(|module| match module.try_borrow() {
                Ok(module) => {
                    if module.id() == id {
                        Some(Ref::map(module, |x| x.deref()))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
    }

    fn module_by_id_mut<'a>(&'a self, id: ModuleId) -> Option<RefMut<'a, dyn Module>> {
        self.modules
            .iter()
            .find_map(|module| match module.try_borrow_mut() {
                Ok(module) => {
                    if module.id() == id {
                        Some(RefMut::map(module, |x| {
                            let x = x.deref_mut();
                            x
                        }))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
