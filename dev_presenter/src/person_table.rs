use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::person::PersonId;
use std::collections::BTreeMap;

pub struct PersonRecord {
    pub name: String,
}

pub struct PersonTable {
    data: BTreeMap<PersonId, PersonRecord>,
}

impl PersonTable {
    pub fn new(environment: &Environment) -> Self {
        let mut data = BTreeMap::new();

        for vessel in environment.vessels() {
            for module in vessel.modules() {
                for person in module.persons() {
                    let ok = data
                        .insert(
                            person.id(),
                            PersonRecord {
                                name: person.name().to_string(),
                            },
                        )
                        .is_none();
                    assert!(ok);
                }
            }
        }

        Self { data }
    }

    pub fn get(&self, id: &PersonId) -> Option<&PersonRecord> {
        self.data.get(id)
    }
}
