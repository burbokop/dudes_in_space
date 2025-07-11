use std::cell::{RefCell, RefMut};
use crate::bl::modules::{Module, ModuleCapability, ModuleSerializerDeserializerRegistry, ModuleVisitor, VesselPersonInterface};
use crate::bl::utils::math::Point;
use crate::bl::utils::utils::Float;
use crate::bl::Person;
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Formatter;

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
pub(crate) struct Vessel {
    id: VesselId,
    pos: Point<Float>,
    modules: Vec<RefCell<Box<dyn Module>>>,
}

pub(crate) struct VesselCreateInfo {
    pub(crate)    pos: Point<Float>,
    pub(crate)    modules: Vec<Box<dyn Module>>
}

impl Vessel {
    pub(crate) fn visit_modules(&self, visitor: &dyn ModuleVisitor<Result=()>) -> Option<()> {
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
        Self {id, pos:ci.pos, modules: ci.modules.into_iter().map(RefCell::new).collect() }
    }
    
    pub(crate) fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(RefCell::new(module));   
    }

    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
        reg: &ModuleSerializerDeserializerRegistry,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ModuleImpl<'b> {
            reg: &'b ModuleSerializerDeserializerRegistry,
        }

        impl<'b, 'de> DeserializeSeed<'de> for ModuleImpl<'b> {
            type Value = Box<dyn Module>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                self.reg.deserialize(deserializer)
            }
        }

        struct ModuleSeqVisitor<'b> {
            reg: &'b ModuleSerializerDeserializerRegistry,
        }

        impl<'b, 'de> Visitor<'de> for ModuleSeqVisitor<'b> {
            type Value = Vec<Box<dyn Module>>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                todo!()
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut modules: Vec<Box<dyn Module>> = Default::default();
                while let Some(key) = seq.next_element_seed(ModuleImpl{ reg: self.reg })? {
                    modules.push(key);
                }
                Ok(modules)
            }
        }


        struct ModuleSeqSeed<'b> {
            reg: &'b ModuleSerializerDeserializerRegistry,
        }

        impl<'b, 'de> DeserializeSeed<'de> for ModuleSeqSeed<'b> {
            type Value = Vec<Box<dyn Module>>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>
            {
                deserializer.deserialize_seq(ModuleSeqVisitor{reg: self.reg})
            }
        }

        enum VesselField {Id, Pos, Modules }

        impl<'de> Deserialize<'de> for VesselField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = VesselField;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`id`, `pos` or `modules`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<VesselField, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "id" => Ok(VesselField::Id),
                            "pos" => Ok(VesselField::Pos),
                            "modules" => Ok(VesselField::Modules),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct VesselVisitor<'b> {
            reg: &'b ModuleSerializerDeserializerRegistry,
        }

        impl<'b,'de> Visitor<'de> for VesselVisitor<'b>{
            type Value = Vessel;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Vessel")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut pos = None;
                let mut modules = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        VesselField::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        },
                        VesselField::Pos => {
                            if pos.is_some() {
                                return Err(de::Error::duplicate_field("pos"));
                            }
                            pos = Some(map.next_value()?);
                        }
                        VesselField::Modules => {
                            if modules.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            modules = Some(map.next_value_seed(ModuleSeqSeed{ reg: self.reg })?);
                        },
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let pos = pos.ok_or_else(|| de::Error::missing_field("pos"))?;
                let modules = modules.ok_or_else(|| de::Error::missing_field("modules"))?;
                Ok(Vessel { id, pos, modules: modules.into_iter().map(RefCell::new).collect()})
            }
        }

        const FIELDS: &[&str] = &["pos", "modules"];

        deserializer.deserialize_struct("Vessel",FIELDS, VesselVisitor{reg })
    }

    pub(crate) fn proceed(&mut self) {
        for v in & self.modules {
            v.borrow_mut().proceed(self)
        }
    }
}

impl VesselPersonInterface for Vessel {
    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<RefMut<Box<dyn Module>>> {
        self.modules.iter().filter_map(|module| {
            if let Ok(module) = module.try_borrow_mut() {
                if module.capabilities().contains(&cap) {
                    return Some(module);
                }
            }
            None
        }).collect()
    }
}

impl Serialize for Vessel {
    fn serialize<'a, S>(&'a self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        struct ModuleImpl<'a> {
            module: &'a RefCell<Box<dyn Module>>,
        }

        impl<'a> Serialize for ModuleImpl<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                ModuleSerializerDeserializerRegistry::serialize(serializer, self.module)
            }
        }

        #[derive(Serialize)]
        struct Impl<'a> {
            id: VesselId,
            pos: Point<Float>,
            modules: Vec<ModuleImpl<'a>>,
        }

        Impl::<'a> {
            id: self.id,
            pos: self.pos,
            modules: self
                .modules
                .iter()
                .map(|module| ModuleImpl { module })
                .collect(),
        }
            .serialize(serializer)
    }
}
