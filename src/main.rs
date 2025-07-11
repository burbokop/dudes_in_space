#![feature(substr_range)]

use std::env::home_dir;
use std::path::Path;
use crate::bl::Environment;
use crate::bl::modules::{Assembler, AssemblerDeserializer, Module, ModuleVisitor, PersonnelArea, PersonnelAreaSerializerDeserializer};
use rand::rng;
use serde::Serialize;
use crate::bl::utils::dyn_serde::DynDeserializeFactoryRegistry;
use crate::bl::VesselModule::Assembly;

mod bl;
mod env_presets;

fn env_from_json(
    registry: &DynDeserializeFactoryRegistry<dyn Module>,
    bytes: &[u8],
) -> Result<Environment, serde_json::Error> {
    let read = serde_json::de::SliceRead::new(bytes);
    let mut de = serde_json::Deserializer::new(read);
    let value = Environment::deserialize(&mut de, registry)?;
    de.end()?;
    Ok(value)
}

fn env_to_json(
    env: &Environment,
    registry: &DynDeserializeFactoryRegistry<dyn Module>,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut writer = Vec::with_capacity(128);
    let mut ser = serde_json::Serializer::new(&mut writer);
    env.serialize(&mut ser).unwrap();
    Ok(writer)
}

fn main() {
    let save_path = home_dir().unwrap().join(".dudes_in_space/save.json");

    let module_serializer_deserializer_registry = DynDeserializeFactoryRegistry::<dyn Module>::new()
        .with(PersonnelAreaSerializerDeserializer)
        .with(AssemblerDeserializer);

    let mut environment = if save_path.exists() {
        env_from_json(
            &module_serializer_deserializer_registry,
            &std::fs::read(save_path.as_path()).unwrap(),
        )
        .unwrap()
    } else {
        env_presets::preset0::new(&mut rng())
    };

    struct MyAssVisitor;
    impl ModuleVisitor for MyAssVisitor {
        type Result = ();
        fn visit_assembler(&self, assembler: &Assembler) -> Option<Self::Result> {
            
            Some(())
            // assembler.
        }
    }

    // environment.vessel_by_id_mut(0).unwrap().visit_modules(&MyAssVisitor);
    environment.proceed();

    println!("{:#?}", environment);


    std::fs::create_dir_all(save_path.parent().unwrap()).unwrap();
    std::fs::write(
        save_path,
        env_to_json(&environment, &module_serializer_deserializer_registry).unwrap(),
    )
    .unwrap();
}
