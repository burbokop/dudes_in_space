use std::cell::RefMut;
use serde::Serialize;
use dyn_serde::{DynDeserializeSeedVault, VecSeed};
use dyn_serde_macro::DeserializeSeedXXX;
use crate::modules::{AssemblyRecipeSeed, Module, ModuleCapability, ModuleFactory, ModuleSeed};

#[derive(Debug, Serialize, DeserializeSeedXXX, Default)]
#[deserialize_seed_xxx(seed = crate::modules::module_storage::ModuleStorageSeed::<'v>)]
pub struct ModuleStorage {
    #[deserialize_seed_xxx(seed = self.seed.content_seed)]
    content: Vec<Box<dyn Module>>,
}

impl ModuleStorage {
    pub(crate) fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    /// TODO: add capacity
    pub fn has_space(&self) -> bool {
        true
    }

    pub fn add(&mut self, module: Box<dyn Module>) -> bool {
        if !self.has_space() {
            return false;       
        }
        
        self.content.push(module);
        true
    }
    
    pub(crate) fn contains_modules_with_cap(&self, cap: ModuleCapability) -> bool {
        self.content.iter().any(|module| module.capabilities().contains(&cap))
    }

    fn modules_with_cap(&self, cap: ModuleCapability) -> Vec<&Box<dyn Module>> {
        self.content
            .iter()
            .filter_map(|module| {
                    if module.capabilities().contains(&cap) {
                        return Some(module);
                    }
                None
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct ModuleStorageSeed<'v> {
    content_seed: VecSeed<ModuleSeed<'v>>,
}

impl<'v> ModuleStorageSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> Self {
        Self {
            content_seed: VecSeed::new(ModuleSeed::new(vault)),
        }
    }
}