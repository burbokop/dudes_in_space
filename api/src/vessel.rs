use crate::PersonId;
use crate::modules::{Module, ModuleCapability, ModuleId, ModuleSeed, VesselModuleInterface, VesselPersonInterface};
use crate::person::Person;
use crate::utils::math::Point;
use crate::utils::utils::Float;
use dyn_serde::DynDeserializeSeedVault;
use dyn_serde_macro::DeserializeSeedXXX;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Formatter;
use std::ops::Deref;

pub(crate) enum VesselModule {
    Cockpit { captain: Option<Person> },
    Cargo,
    FuelTank,
    Radar { navigator: Option<Person> },
    Engine,
    DockingPort,
    ProjectileGun,
    MissileRack,
    WarpDrive,
    Radiators,
    Reactor { engineer: Option<Person> },
    Workshop { worker: Option<Person> },
    Assembly { worker: Option<Person> },
}

pub(crate) type VesselId = u32;

#[derive(Debug)]
enum VesselRequest {
    MoveToModule {
        person_id: PersonId,
        module_id: ModuleId,
    },
    AddModule {
        module: Box<dyn Module>,
    }
}

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::vessel::VesselSeed::<'v>)]
pub struct Vessel {
    id: VesselId,
    pos: Point<Float>,
    #[deserialize_seed_xxx(seed = self.seed.module_seq_seed)]
    modules: Vec<RefCell<Box<dyn Module>>>,
    #[serde(skip)]
    requests: RefCell<Vec<VesselRequest>>,
}

pub struct VesselCreateInfo {
    pub pos: Point<Float>,
    pub modules: Vec<Box<dyn Module>>,
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

impl<'r> VesselSeed<'r> {
    pub(crate) fn new(reg: &'r DynDeserializeSeedVault<dyn Module>) -> Self {
        Self {
            module_seq_seed: ModuleSeqSeed {
                module_seed: ModuleSeed::new(reg),
            },
        }
    }
}

impl Vessel {
    pub(crate) fn id(&self) -> VesselId {
        self.id
    }

    pub(crate) fn new(id: VesselId, ci: VesselCreateInfo) -> Self {
        Self {
            id,
            pos: ci.pos,
            modules: ci.modules.into_iter().map(RefCell::new).collect(),
            requests: Default::default(),
        }
    }

    pub fn modules<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Box<dyn Module>>> {
        self.modules.iter().map(|module| module.borrow())
    }

    pub fn modules_mut<'a>(&'a self) -> impl Iterator<Item = RefMut<'a, Box<dyn Module>>> {
        self.modules.iter().map(|module| module.borrow_mut())
    }

    pub(crate) fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(RefCell::new(module));
    }

    pub(crate) fn proceed(&mut self) {
        for v in &self.modules {
            v.borrow_mut().proceed(self)
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
                    assert!(dst.can_insert_person());
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
        self.requests.borrow_mut().push(VesselRequest::AddModule { module })
    }

    fn vessel_person_interface(&self) -> &dyn VesselPersonInterface {
        self
    }
}

impl VesselPersonInterface for Vessel {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>> {
        self.modules
            .iter()
            .filter_map(|module| {
                if let Ok(module) = module.try_borrow_mut() {
                    if module.capabilities().contains(&cap) {
                        return Some(module);
                    }
                }
                None
            })
            .collect()
    }

    fn move_to_module(&self, person_id: PersonId, id: ModuleId) {
        self.requests
            .borrow_mut()
            .push(VesselRequest::MoveToModule {
                person_id,
                module_id: id,
            })
    }
}
