use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::person::PersonId;
use dudes_in_space_api::vessel::VesselId;
use std::collections::BTreeMap;
use std::ops::ControlFlow;

pub struct VesselRecord {
    pub name: String,
}

pub struct VesselTable {
    data: BTreeMap<VesselId, VesselRecord>,
}

impl VesselTable {
    pub fn new(environment: &Environment) -> Self {
        let mut data = BTreeMap::new();

        for vessel in environment.vessels() {
            let _: ControlFlow<()> = vessel.traverse(|_, vessel| {
                let ok = data
                    .insert(
                        vessel.id(),
                        VesselRecord {
                            name: vessel.name().to_string(),
                        },
                    )
                    .is_none();
                assert!(ok);

                ControlFlow::Continue(())
            });
        }

        Self { data }
    }

    pub fn get(&self, id: &PersonId) -> Option<&VesselRecord> {
        self.data.get(id)
    }
}
