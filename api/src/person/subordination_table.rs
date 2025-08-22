use crate::person::PersonId;
use crate::vessel::VesselConsole;
use std::cell::RefCell;
use std::collections::BTreeMap;

type SubordinationChain = Vec<PersonId>;
type SubordinationChainRef<'a> = &'a [PersonId];

pub struct SubordinationTable {
    // TODO use a cache
    subordination_chains: RefCell<BTreeMap<PersonId, SubordinationChain>>,
    bosses: BTreeMap<PersonId, PersonId>,
}

pub enum VesselPermissions {
    Enter,
    Operate,
    Pilot,
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
        permission: VesselPermissions,
    ) -> bool {
        // TODO: take into account permissions
        self.subordination_chain(person_id)
            .contains(&vessel.owner())
    }
}
