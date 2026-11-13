use crate::module::{Module, ModuleCapability, ModuleId, ModuleSeed};
use dyn_serde::{DynDeserializeSeedVault, VecSeed};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize, DeserializeSeedXXX, Default)]
#[deserialize_seed_xxx(seed = crate::module::ModuleStorageSeed::<'v>)]
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

    pub fn content(&self) -> &[Box<dyn Module>] {
        &self.content
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

    pub fn contains_all<'a>(&self, mut ids: impl Iterator<Item = &'a ModuleId>) -> bool {
        ids.all(|id| self.content.iter().any(|module| module.id() == *id))
    }

    pub fn try_take<'a>(
        &mut self,
        ids: impl Iterator<Item = &'a ModuleId> + Clone,
    ) -> Option<Vec<Box<dyn Module>>> {
        if !self.contains_all(ids.clone()) {
            return None;
        }

        let mut result: Vec<Box<dyn Module>> = Default::default();
        for id in ids {
            result.push(
                self.content
                    .extract_if(.., |module| module.id() == *id)
                    .next()
                    .unwrap(),
            )
        }
        Some(result)
    }

    pub fn contains_modules_with_capability(&self, cap: ModuleCapability) -> bool {
        self.content
            .iter()
            .any(|module| module.capabilities().contains(&cap))
    }

    pub fn contains_modules_with_primary_capability(&self, cap: ModuleCapability) -> bool {
        self.content
            .iter()
            .any(|module| module.primary_capabilities().contains(&cap))
    }

    fn modules_with_capability(&self, cap: ModuleCapability) -> Vec<&Box<dyn Module>> {
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

    fn modules_with_primary_capability(&self, cap: ModuleCapability) -> Vec<&Box<dyn Module>> {
        self.content
            .iter()
            .filter_map(|module| {
                if module.primary_capabilities().contains(&cap) {
                    return Some(module);
                }
                None
            })
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Module> {
        self.content.iter().map(|module| module.deref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Module> {
        self.content.iter_mut().map(|module| {
            let x = module.deref_mut();
            x
        })
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
