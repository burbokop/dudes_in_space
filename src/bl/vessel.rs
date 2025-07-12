use crate::bl::Person;
use crate::bl::modules::{ModuleSeed,
    Module, ModuleCapability, ModuleVisitor, VesselPersonInterface,
};
use crate::bl::utils::dyn_serde::DynDeserializeSeedVault;
use crate::bl::utils::math::Point;
use crate::bl::utils::utils::Float;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::cell::{RefCell, RefMut};
use std::fmt::Formatter;
use dudes_in_space_macro::DeserializeSeedXXX;
use sdl2::libc::write;

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

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::bl::vessel::VesselSeed::<'v>)]
pub struct Vessel {
    id: VesselId,
    pos: Point<Float>,
    #[deserialize_seed_xxx(seed = self.seed.module_seq_seed)]
    modules: Vec<RefCell<Box<dyn Module>>>,
}

pub(crate) struct VesselCreateInfo {
    pub(crate) pos: Point<Float>,
    pub(crate) modules: Vec<Box<dyn Module>>,
}

#[derive(Clone)]
struct ModuleSeqSeed<'v> {
    module_seed: ModuleSeed<'v>,
}

impl<'de, 'v> DeserializeSeed<'de> for ModuleSeqSeed<'v> {
    type Value = Vec<RefCell<Box<dyn Module>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>
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
        
        deserializer.deserialize_seq(ModuleSeqVisitor { module_seed: self.module_seed })
    }
}

#[derive(Clone)]
pub(crate) struct VesselSeed<'v> {
    module_seq_seed: ModuleSeqSeed<'v>,
}

impl<'r> VesselSeed<'r> {
    pub(crate) fn new(reg: &'r DynDeserializeSeedVault<dyn Module>) -> Self {
        Self { module_seq_seed: ModuleSeqSeed { module_seed: ModuleSeed::new(reg) } }
    }
}

impl Vessel {
    pub(crate) fn visit_modules(&self, visitor: &dyn ModuleVisitor<Result = ()>) -> Option<()> {
        for m in &self.modules {
            if let Some(r) = m.borrow().accept_visitor(visitor) {
                return Some(r);
            }
        }
        None
    }

    pub(crate) fn id(&self) -> VesselId {
        self.id
    }

    pub(crate) fn new(id: VesselId, ci: VesselCreateInfo) -> Self {
        Self {
            id,
            pos: ci.pos,
            modules: ci.modules.into_iter().map(RefCell::new).collect(),
        }
    }

    pub(crate) fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(RefCell::new(module));
    }

    pub(crate) fn proceed(&mut self) {
        for v in &self.modules {
            v.borrow_mut().proceed(self)
        }
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
}
