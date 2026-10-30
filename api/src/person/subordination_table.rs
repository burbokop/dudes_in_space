use crate::person::PersonId;
use crate::vessel::VesselConsole;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;

type SubordinationChain = Vec<PersonId>;
type SubordinationChainRef<'a> = &'a [PersonId];

pub struct SubordinationTable {
    // TODO use a cache
    subordination_chains: RefCell<BTreeMap<PersonId, SubordinationChain>>,
    bosses: BTreeMap<PersonId, PersonId>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum VesselPermission {
    None,
    Enter,
    Operate,
    Pilot,
}

impl VesselPermission {
    pub(crate) fn has_permission(&self, permission: VesselPermission) -> bool {
        *self as u8 >= permission as u8
    }
}

impl Default for VesselPermission {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VesselPermissions {
    pub subordinate: VesselPermission,
    pub other: VesselPermission,
}

impl SubordinationTable {
    pub fn new() -> Self {
        Self {
            subordination_chains: Default::default(),
            bosses: BTreeMap::new(),
        }
    }

    pub fn link(&mut self, subordinate: PersonId, boss: Option<PersonId>) {
        if let Some(boss) = boss {
            self.bosses.insert(subordinate, boss);
        } else {
            self.bosses.remove(&subordinate);
        }
    }

    fn build_subordination_chain(&self, person_id: PersonId) -> SubordinationChain {
        let mut chain: SubordinationChain = Default::default();
        chain.push(person_id);
        while let Some(boss) = self.bosses.get(&chain.last().unwrap()) {
            chain.push(*boss);
        }
        chain.reverse();
        chain
    }

    fn subordination_chain(&self, person_id: PersonId) -> SubordinationChain {
        // TODO use subordination_chains instead
        self.build_subordination_chain(person_id)
    }

    pub fn has_permission(
        &self,
        person_id: PersonId,
        vessel: &dyn VesselConsole,
        permission: VesselPermission,
    ) -> bool {
        if person_id == vessel.owner() {
            return true;
        }

        if self
            .subordination_chain(person_id)
            .contains(&vessel.owner())
        {
            return vessel.permissions().subordinate.has_permission(permission);
        }

        vessel.permissions().other.has_permission(permission)
    }
}
