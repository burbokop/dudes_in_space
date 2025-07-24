use dudes_in_space_api::environment::{Environment, EnvironmentSeed};
use dudes_in_space_api::module::Module;
use dudes_in_space_core::modules::VisitModules;
use dyn_serde::DynDeserializeSeedVault;
use rand::rng;
use serde::Serialize;
use serde::de::DeserializeSeed;
use std::env::home_dir;

mod env_presets;

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
    env.serialize(&mut ser).unwrap();
    Ok(writer)
}

fn main() {
    let save_path = home_dir().unwrap().join(".dudes_in_space/save.json");

    let module_factory_seed_vault =
        dudes_in_space_core::register_module_factories(Default::default()).into_rc();
    let module_seed_vault =
        dudes_in_space_core::register_modules(Default::default(), module_factory_seed_vault)
            .into_rc();

    let mut environment = if save_path.exists() {
        env_from_json(
            &module_seed_vault,
            &std::fs::read(save_path.as_path()).unwrap(),
        )
        .unwrap()
    } else {
        env_presets::preset0::new(&mut rng())
    };

    // struct MyAssVisitor;
    // impl ModuleVisitorMut for MyAssVisitor {
    //     type Result = ();
    //     fn visit_assembler(&self, assembler: &mut Assembler) -> Option<Self::Result> {
    //         // assembler.add_recipe(AssemblyRecipe::new(
    //         //     vec![Item {
    //         //         id: "steel".to_string(),
    //         //         count: 10,
    //         //     }]
    //         //     .into(),
    //         //     todo!(),
    //         // ));
    //         Some(())
    //     }
    // }

    // environment.vessel_by_id_mut(0).unwrap().visit_modules_mut(&MyAssVisitor);
    environment.proceed();

    // println!("{:#?}", environment);

    std::fs::create_dir_all(save_path.parent().unwrap()).unwrap();
    std::fs::write(save_path, env_to_json(&environment).unwrap()).unwrap();
}
