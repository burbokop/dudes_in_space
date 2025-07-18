use crate::Vessel;
use crate::modules::Module;
use crate::vessel::{VesselCreateInfo, VesselId, VesselSeed};
use dyn_serde::DynDeserializeSeedVault;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Serialize)]
pub struct Environment {
    vessels: Vec<Vessel>,
    next_vessel_id: VesselId,
}

pub struct EnvironmentSeed<'r> {
    reg: &'r DynDeserializeSeedVault<dyn Module>,
}

impl<'r> EnvironmentSeed<'r> {
    pub fn new(reg: &'r DynDeserializeSeedVault<dyn Module>) -> Self {
        Self { reg }
    }
}

impl<'de, 'r> DeserializeSeed<'de> for EnvironmentSeed<'r> {
    type Value = Environment;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VesselSeqVisitor<'b> {
            reg: &'b DynDeserializeSeedVault<dyn Module>,
        }

        impl<'b, 'de> Visitor<'de> for VesselSeqVisitor<'b> {
            type Value = Vec<Vessel>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                todo!()
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vessels: Vec<Vessel> = Default::default();
                while let Some(key) = seq.next_element_seed(VesselSeed::new(self.reg))? {
                    vessels.push(key);
                }
                Ok(vessels)
            }
        }

        struct VesselSeqSeed<'b> {
            reg: &'b DynDeserializeSeedVault<dyn Module>,
        }

        impl<'b, 'de> DeserializeSeed<'de> for VesselSeqSeed<'b> {
            type Value = Vec<Vessel>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_seq(VesselSeqVisitor { reg: self.reg })
            }
        }

        enum EnvironmentField {
            NextVesselId,
            Vessels,
        }

        impl<'de> Deserialize<'de> for EnvironmentField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = EnvironmentField;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`next_vessel_id` or `vessels`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<EnvironmentField, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "next_vessel_id" => Ok(EnvironmentField::NextVesselId),
                            "vessels" => Ok(EnvironmentField::Vessels),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct EnvironmentVisitor<'b> {
            reg: &'b DynDeserializeSeedVault<dyn Module>,
        }

        impl<'b, 'de> Visitor<'de> for EnvironmentVisitor<'b> {
            type Value = Environment;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Environment")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut next_vessel_id = None;
                let mut vessels = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        EnvironmentField::NextVesselId => {
                            if next_vessel_id.is_some() {
                                return Err(de::Error::duplicate_field("pos"));
                            }
                            next_vessel_id = Some(map.next_value()?);
                        }
                        EnvironmentField::Vessels => {
                            if vessels.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            vessels = Some(map.next_value_seed(VesselSeqSeed { reg: self.reg })?);
                        }
                    }
                }
                let next_vessel_id =
                    next_vessel_id.ok_or_else(|| de::Error::missing_field("pos"))?;
                let vessels = vessels.ok_or_else(|| de::Error::missing_field("modules"))?;
                Ok(Environment {
                    vessels,
                    next_vessel_id,
                })
            }
        }

        const FIELDS: &[&str] = &["next_vessel_id", "vessels"];

        deserializer.deserialize_struct("Environment", FIELDS, EnvironmentVisitor { reg: self.reg })
    }
}

impl Environment {
    pub fn new(vessels: Vec<VesselCreateInfo>) -> Self {
        let mut next_vessel_id = 0;
        let vessels = vessels
            .into_iter()
            .map(|ci| {
                let v = Vessel::new(next_vessel_id, ci);
                next_vessel_id += 1;
                v
            })
            .collect();
        Self {
            vessels,
            next_vessel_id,
        }
    }

    pub(crate) fn add(&mut self, vessel: VesselCreateInfo) {
        self.vessels.push(Vessel::new(self.next_vessel_id, vessel));
        self.next_vessel_id += 1;
    }

    pub(crate) fn vessel_by_id(&self, id: VesselId) -> Option<&Vessel> {
        self.vessels.iter().find(|v| v.id() == id)
    }

    pub fn vessel_by_id_mut(&mut self, id: VesselId) -> Option<&mut Vessel> {
        self.vessels.iter_mut().find(|v| v.id() == id)
    }

    pub fn proceed(&mut self) {
        for v in &mut self.vessels {
            v.proceed()
        }
    }
}
