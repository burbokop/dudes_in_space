use crate::components::Components;
use dudes_in_space_api::environment::{Environment, EnvironmentSeed};
use dudes_in_space_api::module::Module;
use dudes_in_space_core::env_presets;
use dyn_serde::DynDeserializeSeedVault;
use rand::rng;
use serde::Serialize;
use serde::de::DeserializeSeed;
use std::path::PathBuf;

fn env_from_json(
    registry: &DynDeserializeSeedVault<dyn Module>,
    bytes: &[u8],
) -> Result<Environment, serde_json::Error> {
    let read = serde_json::de::SliceRead::new(bytes);
    let mut de = serde_json::Deserializer::new(read);
    let value = EnvironmentSeed::new(registry).deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

fn env_to_json(env: &Environment) -> Result<Vec<u8>, serde_json::Error> {
    let mut writer = Vec::with_capacity(128);
    let mut ser = serde_json::Serializer::pretty(&mut writer);
    env.serialize(&mut ser)?;
    Ok(writer)
}

pub(crate) fn load(components: &Components, save_path: PathBuf) -> Environment {
    if save_path.exists() {
        env_from_json(
            &components.module_seed_vault,
            &std::fs::read(save_path.as_path()).unwrap(),
        )
        .unwrap()
    } else {
        env_presets::preset0::new(&mut rng(), &components.item_vault)
    }
}

pub(crate) fn save(environment: Environment, save_path: PathBuf) {
    std::fs::create_dir_all(save_path.parent().unwrap()).unwrap();
    std::fs::write(save_path, env_to_json(&environment).unwrap()).unwrap();
}
