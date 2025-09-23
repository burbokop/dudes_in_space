use crate::item::{ItemStorage, ItemVolume};
use crate::module::{ConcatModuleCapabilities, Module, ModuleCapability, ModuleConsole, ModuleId};
use crate::recipe::{AssemblyRecipe, InputItemRecipe, ItemRecipe, ItemRecipeHash, OutputItemRecipe};
use crate::utils::physics::M3;
use crate::vessel::{DockingClamp, DockingClampConnection, VesselConsole, VesselInternalConsole};
use std::cell::Ref;
use std::collections::BTreeSet;
use std::ops::{Deref, Try};

pub enum MaybeCellRef<'a, 'b, T> {
    PlainRef(&'a T),
    CellRef(Ref<'b, T>),
}

impl<'a, 'b, T> Deref for MaybeCellRef<'a, 'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeCellRef::PlainRef(r) => *r,
            MaybeCellRef::CellRef(r) => r.deref(),
        }
    }
}

pub struct ThisVessel<'a, 'b> {
    pub(crate) this_module: &'a mut dyn ModuleConsole,
    pub(crate) this_vessel: &'b dyn VesselInternalConsole,
}

pub fn tie<'a, 'b>(
    this_module: &'a mut dyn ModuleConsole,
    this_vessel: &'b dyn VesselInternalConsole,
) -> ThisVessel<'a, 'b> {
    ThisVessel {
        this_module,
        this_vessel,
    }
}

pub struct ForEachDockingClampsEntry<'d, 'm> {
    pub clamp: &'d DockingClamp,
    pub module: Option<&'m dyn Module>,
}

impl<'a, 'b> ThisVessel<'a, 'b> {
    pub fn capabilities(&self) -> BTreeSet<ModuleCapability> {
        self.this_vessel
            .capabilities()
            .concat(self.this_module.capabilities())
    }

    pub fn primary_capabilities(&self) -> BTreeSet<ModuleCapability> {
        self.this_vessel
            .primary_capabilities()
            .concat(self.this_module.primary_capabilities())
    }

    pub fn has_capabilities(
        &self,
        needed_caps: impl IntoIterator<Item = ModuleCapability>,
    ) -> bool {
        let this_vessel_caps = self.capabilities();
        needed_caps
            .into_iter()
            .all(|cap| this_vessel_caps.contains(&cap))
    }

    pub fn has_primary_capabilities(
        &self,
        needed_caps: impl IntoIterator<Item = ModuleCapability>,
    ) -> bool {
        let this_vessel_caps = self.primary_capabilities();
        needed_caps
            .into_iter()
            .all(|cap| this_vessel_caps.contains(&cap))
    }

    pub fn for_each_docking_clamps_with_vessel_which_has_caps<F, R>(
        &self,
        caps: &[ModuleCapability],
        primary_caps: &[ModuleCapability],
        f: F,
    ) -> R
    where
        F: FnMut(ForEachDockingClampsEntry) -> R,
        R: Try<Output = ()>,
    {
        self.this_vessel
            .modules_with_capability(ModuleCapability::DockingClamp)
            .iter()
            .map(|m| m.docking_clamps().iter().map(|x| (x, Some(m.deref()))))
            .flatten()
            .chain(self.this_module.docking_clamps().iter().map(|x| (x, None)))
            .filter_map(|(clamp, module)| {
                clamp.connection().and_then(
                    |DockingClampConnection {
                         vessel,
                         connector_id,
                     }| {
                        let vessel_capabilities = vessel.capabilities();
                        let vessel_primary_capabilities = vessel.primary_capabilities();
                        (caps.iter().all(|cap| vessel_capabilities.contains(&cap))
                            && primary_caps
                                .iter()
                                .all(|cap| vessel_primary_capabilities.contains(&cap)))
                        .then_some(ForEachDockingClampsEntry { clamp, module })
                    },
                )
            })
            .try_for_each(f)
    }

    pub fn find_item_recipe(
        &self,
        recipe: ItemRecipeHash,
    ) -> Option<(ModuleId, ItemRecipe)> {
        self.this_vessel
            .modules_with_capability(ModuleCapability::ItemCrafting)
            .iter()
            .map(|m| m.item_recipes().iter().map(|c| (m.id(), c)))
            .flatten()
            .chain(
                self.this_module
                    .crafting_console().map(|x|x.item_recipes().iter()).unwrap_or([].iter())
                    .map(|c| (self.this_module.id(), c)),
            )
            .find_map(|(a, b)|
                if b.default_hash() == recipe {
                    Some((a,b.clone())
                    )
                }else {None})
    }

    pub fn find_map_docking_clamp<T>(
        &self,
        mut f: impl FnMut(ModuleId, &DockingClamp) -> Option<T>,
    ) -> Option<T> {
        self.this_vessel
            .modules_with_capability(ModuleCapability::DockingClamp)
            .iter()
            .map(|m| m.docking_clamps().iter().map(|c| (m.id(), c)))
            .flatten()
            .chain(
                self.this_module
                    .docking_clamps()
                    .iter()
                    .map(|c| (self.this_module.id(), c)),
            )
            .find_map(|(a, b)| f(a, b))
    }

    pub fn find_map_docking_clamp_mut<T>(
        &mut self,
        mut f: impl FnMut(ModuleId, &mut DockingClamp) -> Option<T>,
    ) -> Option<T> {
        let this_module_id = self.this_module.id();
        self.this_vessel
            .modules_with_capability_mut(ModuleCapability::DockingClamp)
            .iter_mut()
            .map(|m| {
                let id = m.id();
                m.docking_clamps_mut().iter_mut().map(move |c| (id, c))
            })
            .flatten()
            .chain(
                self.this_module
                    .docking_clamps_mut()
                    .iter_mut()
                    .map(|c| (this_module_id, c)),
            )
            .find_map(|(a, b)| f(a, b))
    }

    pub fn total_primary_free_space(&self) -> ItemVolume {
        self.this_vessel
            .modules_with_primary_capability(ModuleCapability::ItemStorage)
            .iter()
            .map(|module| {
                module
                    .storages()
                    .iter()
                    .map(|storage| storage.free_space())
                    .sum::<ItemVolume>()
            })
            .sum::<ItemVolume>()
            + if self
                .this_module
                .primary_capabilities()
                .contains(&ModuleCapability::ItemStorage)
            {
                self.this_module
                    .storages()
                    .iter()
                    .map(|storage| storage.free_space())
                    .sum::<ItemVolume>()
            } else {
                M3(0)
            }
    }

    pub fn storages<'q>(&'q self) -> Vec<MaybeCellRef<'a, 'b, ItemStorage>>
    where
        'q: 'a + 'b,
    {
        // struct Iter<'a, 'b> {
        //     borrowed: std::vec::IntoIter<Ref<'a, dyn Module>> ,
        //
        //     this: std::slice::Iter<'b, ItemStorage>,
        // }
        //
        // impl<'a, 'b> Iterator for Iter<'a, 'b> {
        //     type Item = MaybeCellRef<'a, 'b, ItemStorage>;
        //
        //     fn next(&mut self) -> Option<Self::Item> {
        //         todo!()
        //     }
        // }
        //
        // Iter {
        //     borrowed: self.this_vessel
        //         .modules_with_capability(ModuleCapability::ItemStorage).into_iter(),
        //     this: self.this_module.storages().into_iter()
        // }

        let mut xx: Vec<MaybeCellRef<'a, 'b, ItemStorage>> = self
            .this_module
            .storages()
            .into_iter()
            .map(|x: &'a ItemStorage| MaybeCellRef::<'a, 'b, ItemStorage>::PlainRef(x))
            .collect();

        let modules = self
            .this_vessel
            .modules_with_capability(ModuleCapability::ItemStorage);

        let mut yy: Vec<MaybeCellRef<'a, 'b, ItemStorage>> = modules
            .iter()
            .map(|module| {
                let c = module.storages().len();
                (0..c).map(|i| Ref::map(Ref::clone(module), |module| module.storages()[i]))
            })
            .flatten()
            .map(|x| MaybeCellRef::<'a, 'b, ItemStorage>::CellRef(x))
            .collect();

        xx.append(&mut yy);
        xx
    }

    pub fn item_recipes(&self) -> Vec<ItemRecipe> {
        self.this_vessel
            .modules_with_capability(ModuleCapability::ItemCrafting)
            .iter()
            .map(|crafter| crafter.item_recipes().iter())
            .chain(
                self.this_module
                    .crafting_console()
                    .iter()
                    .map(|x| x.item_recipes().iter()),
            )
            .flatten()
            .cloned()
            .collect()
    }

    pub fn input_item_recipes(&self) -> Vec<InputItemRecipe> {
        self.this_vessel
            .modules_with_capability(ModuleCapability::ItemConsumption)
            .iter()
            .map(|crafter| crafter.input_item_recipes().iter())
            .chain(
                self.this_module
                    .crafting_console()
                    .iter()
                    .map(|x| x.input_item_recipes().iter()),
            )
            .flatten()
            .cloned()
            .collect()
    }

    pub fn output_item_recipes(&self) -> Vec<OutputItemRecipe> {
        self.this_vessel
            .modules_with_capability(ModuleCapability::ItemProduction)
            .iter()
            .map(|crafter| crafter.output_item_recipes().iter())
            .chain(
                self.this_module
                    .crafting_console()
                    .iter()
                    .map(|x| x.output_item_recipes().iter()),
            )
            .flatten()
            .cloned()
            .collect()
    }

    pub fn assembly_recipes(&self) -> Vec<AssemblyRecipe> {
        self.this_vessel
            .modules_with_capability(ModuleCapability::ModuleCrafting)
            .iter()
            .map(|crafter| crafter.assembly_recipes().iter())
            .chain(
                self.this_module
                    .crafting_console()
                    .iter()
                    .map(|x| x.assembly_recipes().iter()),
            )
            .flatten()
            .cloned()
            .collect()
    }

    pub fn potential_item_recipes(&self) -> Vec<ItemRecipe> {
        self.assembly_recipes()
            .into_iter()
            .map(|r| r.output_description().item_recipes().to_vec())
            .flatten()
            .collect()
    }

    pub fn potential_input_item_recipes(&self) -> Vec<InputItemRecipe> {
        self.assembly_recipes()
            .into_iter()
            .map(|r| r.output_description().input_item_recipes().to_vec())
            .flatten()
            .collect()
    }

    pub fn potential_output_item_recipes(&self) -> Vec<OutputItemRecipe> {
        self.assembly_recipes()
            .into_iter()
            .map(|r| r.output_description().output_item_recipes().to_vec())
            .flatten()
            .collect()
    }

    pub fn potential_assembly_recipes(&self) -> Vec<AssemblyRecipe> {
        self.assembly_recipes()
            .into_iter()
            .map(|r| r.output_description().assembly_recipes().to_vec())
            .flatten()
            .collect()
    }
}
