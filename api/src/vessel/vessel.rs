use crate::module::{Module, ModuleCapability, ModuleId, ModuleSeed, ProcessTokenContext};
use crate::person::{Logger, ObjectiveDeciderVault, PersonId};
use crate::utils::math::Point;
use crate::utils::non_nil_uuid::NonNilUuid;
use crate::utils::utils::Float;
use crate::vessel::{DockingConnectorId, MoveToDockedVesselError, MoveToModuleError, VesselConsole, VesselModuleInterface};
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::{Deserializer, Serialize};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeSet;
use std::fmt::Formatter;
use std::iter;

pub type VesselId = NonNilUuid;

#[derive(Debug)]
enum VesselRequest {
    MoveToModule {
        person_id: PersonId,
        module_id: ModuleId,
    },
    AddModule {
        module: Box<dyn Module>,
    },
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::VesselSeed::<'v>)]
pub struct Vessel {
    id: VesselId,
    owner: PersonId,
    pos: Point<Float>,
    #[deserialize_seed_xxx(seed = self.seed.module_seq_seed)]
    modules: Vec<RefCell<Box<dyn Module>>>,
    #[serde(skip)]
    requests: RefCell<Vec<VesselRequest>>,
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
    pub(crate) fn owner(&self) -> PersonId {
        self.owner
    }

    pub fn new(owner: PersonId, pos: Point<Float>, modules: Vec<Box<dyn Module>>) -> Self {
        Self {
            id: VesselId::new_v4(),
            owner,
            pos,
            modules: modules.into_iter().map(RefCell::new).collect(),
            requests: Default::default(),
        }
    }

    pub fn modules<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Box<dyn Module>>> {
        self.modules.iter().map(|module| module.borrow())
    }

    pub fn modules_mut<'a>(&'a self) -> impl Iterator<Item = RefMut<'a, Box<dyn Module>>> {
        self.modules.iter().map(|module| module.borrow_mut())
    }

    pub fn module_by_id<'a>(&'a self, id: ModuleId) -> Option<Ref<'a, Box<dyn Module>>> {
        self.modules
            .iter()
            .find_map(|module| match module.try_borrow() {
                Ok(module) => {
                    if module.id() == id {
                        Some(module)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
    }

    pub fn module_by_id_mut<'a>(&'a self, id: ModuleId) -> Option<RefMut<'a, Box<dyn Module>>> {
        self.modules
            .iter()
            .find_map(|module| match module.try_borrow_mut() {
                Ok(module) => {
                    if module.id() == id {
                        Some(module)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
    }

    pub fn modules_with_capability<'a>(&'a self, cap: ModuleCapability) -> impl Iterator <Item=Ref<'a, Box<dyn Module>>> {
        self.modules
            .iter()
            .filter_map(move|module| {
                if let Ok(module) = module.try_borrow() {
                    if module.capabilities().contains(&cap) {
                        return Some(module);
                    }
                }
                None
            })
    }

    pub fn modules_with_capability_mut<'a>(&'a self, cap: ModuleCapability) -> impl Iterator <Item=RefMut<'a, Box<dyn Module>>> {
        self.modules
            .iter()
            .filter_map(move|module| {
                if let Ok(module) = module.try_borrow_mut() {
                    if module.capabilities().contains(&cap) {
                        return Some(module);
                    }
                }
                None
            })
    }

    pub fn modules_with_primary_capability<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> impl Iterator <Item=Ref<'a, Box<dyn Module>>> {
        #[allow(unreachable_code)]
        iter::once(todo!())
    }

    pub fn modules_with_primary_capability_mut<'a>(
        &'a self,
        cap: ModuleCapability,
    ) -> impl Iterator <Item=RefMut<'a, Box<dyn Module>>> {
        #[allow(unreachable_code)]
        iter::once(todo!())
    }

    pub(crate) fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(RefCell::new(module));
    }

    pub(crate) fn proceed(
        &mut self,
        process_token_context: &ProcessTokenContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        for v in &self.modules {
            v.borrow_mut()
                .proceed(self, process_token_context, decider_vault, logger)
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
                VesselRequest::AddModule { module } => {
                    self.modules.push(RefCell::new(module));
                }
            }
        }
    }
}

impl VesselModuleInterface for Vessel {
    fn add_module(&self, module: Box<dyn Module>) {
        self.requests
            .borrow_mut()
            .push(VesselRequest::AddModule { module })
    }

    fn owner(&self) -> PersonId {
        self.owner
    }

    fn console(&self) -> &dyn VesselConsole {
        self
    }
}

impl VesselConsole for Vessel {
    fn modules_with_capability<'a>(&'a self, cap: ModuleCapability) -> Vec<Ref<'a, Box<dyn Module>>> {
        self.modules_with_capability(cap).collect()
    }

    fn modules_with_capability_mut<'a>(&'a self, cap: ModuleCapability) -> Vec<RefMut<'a, Box<dyn Module>>> {
        self.modules_with_capability_mut(cap).collect()
    }

    fn modules_with_primary_capability<'a>(&'a self, cap: ModuleCapability) -> Vec<Ref<'a, Box<dyn Module>>> {
        self.modules_with_primary_capability(cap).collect()
    }

    fn modules_with_primary_capability_mut<'a>(&'a self, cap: ModuleCapability) -> Vec<RefMut<'a, Box<dyn Module>>> {
        self.modules_with_primary_capability_mut(cap).collect()
    }

    fn move_person_to_module(
        &self,
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
                    VesselRequest::AddModule { .. } => false,
                }
            })
            .count();

        let module = self.module_by_id(module_id).unwrap();
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

    fn move_person_to_docked_vessel(&self, person_id: PersonId, connector_id: DockingConnectorId) -> Result<(), MoveToDockedVesselError> {
        todo!()
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
}
