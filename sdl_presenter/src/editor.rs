use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::item::{ItemStack, ItemStorage, ItemVault};
use dudes_in_space_api::person::{Awareness, Boldness, Gender, Morale, Passion, Person};
use dudes_in_space_api::utils::math::Point;
use dudes_in_space_api::utils::physics::M3;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::{Vessel, VesselId};
use dudes_in_space_core::{__modules, items};

pub(crate) enum EditorState {
    Selection {
        selected_vessel_id: Option<VesselId>,
    },
    Placing {
        preset_to_place: EditorPreset,
    },
}

#[derive(Debug)]
pub(crate) enum EditorPreset {
    PeterCrafter,
}

impl EditorPreset {
    fn create_vessel(&self, item_vault: &ItemVault, pos: Point<Float>) -> Vessel {
        match self {
            EditorPreset::PeterCrafter => {
                let person = Person::new(
                    "Peter".into(),
                    20,
                    Gender::NonBinary,
                    vec![Passion::Management, Passion::Crafting],
                    Morale::Saint,
                    Boldness::Brave,
                    Awareness::Ascended,
                );

                Vessel::new(
                    "Peters crafting vessel".into(),
                    person.id(),
                    pos,
                    vec![__modules::Assembler::with_operator(
                        person,
                        ItemStorage::from_vec(
                            vec![
                                ItemStack::new(&item_vault, items::STEEL.into(), 10000).unwrap(),
                                ItemStack::new(&item_vault, items::PLASTIC.into(), 10000).unwrap(),
                                ItemStack::new(&item_vault, items::MICROELECTRONICS.into(), 10000)
                                    .unwrap(),
                            ],
                            M3(1000000),
                        )
                        .unwrap(),
                    )],
                )
            }
        }
    }
}

pub(crate) struct Editor {
    state: EditorState,
    nearest_vessel: Option<VesselId>,
    about_to_delete_selected: bool,
    log_lines_count_limit: usize,
}

impl Editor {
    pub(crate) fn new() -> Self {
        Self {
            state: EditorState::Selection {
                selected_vessel_id: None,
            },
            nearest_vessel: None,
            about_to_delete_selected: false,
            log_lines_count_limit: 8,
        }
    }

    pub(crate) fn state(&self) -> &EditorState {
        &self.state
    }

    pub(crate) fn nearest_vessel(&self) -> Option<&VesselId> {
        self.nearest_vessel.as_ref()
    }

    pub(crate) fn about_to_delete_selected(&self) -> bool {
        self.about_to_delete_selected
    }

    pub(crate) fn log_lines_count_limit(&self) -> usize {
        self.log_lines_count_limit
    }

    pub(crate) fn increment_log_lines_count_limit(&mut self) {
        self.log_lines_count_limit += 1;
    }

    pub(crate) fn decrement_log_lines_count_limit(&mut self) {
        if self.log_lines_count_limit > 1 {
            self.log_lines_count_limit -= 1;
        }
    }

    pub(crate) fn selected_vessel(&self) -> Option<&VesselId> {
        match &self.state {
            EditorState::Selection { selected_vessel_id } => selected_vessel_id.as_ref(),
            EditorState::Placing { .. } => None,
        }
    }

    pub(crate) fn delete_selected(&mut self) {
        assert!(self.selected_vessel().is_some());
        self.about_to_delete_selected = true;
    }

    pub(crate) fn confirm_operation(&mut self, environment: &mut Environment) {
        if self.about_to_delete_selected {
            self.about_to_delete_selected = false;
            environment.extract_vessel_by_id(self.selected_vessel().unwrap().clone());
        }
    }

    pub(crate) fn update_nearest_vessel(&mut self, environment: &Environment, pos: Point<Float>) {
        let (nearest_vessel_id, _) = environment
            .vessels()
            .iter()
            .map(|vessel| (vessel.id(), (vessel.pos() - pos).len_sqr()))
            .min_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap())
            .unwrap();
        self.nearest_vessel = Some(nearest_vessel_id)
    }

    pub(crate) fn select_hearest_vessel(&mut self, environment: &mut Environment) {
        self.about_to_delete_selected = false;
        match &mut self.state {
            EditorState::Selection { selected_vessel_id } => {
                *selected_vessel_id = Some(self.nearest_vessel.unwrap());
            }
            EditorState::Placing { .. } => panic!("Cannot select when in placing mode"),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.about_to_delete_selected = false;
        match &mut self.state {
            EditorState::Selection { selected_vessel_id } => {
                *selected_vessel_id = None;
            }
            EditorState::Placing { .. } => {
                self.state = EditorState::Selection {
                    selected_vessel_id: None,
                }
            }
        }
    }

    pub(crate) fn begin_placing(&mut self, preset: EditorPreset) {
        self.about_to_delete_selected = false;
        self.state = EditorState::Placing {
            preset_to_place: preset,
        }
    }

    pub(crate) fn end_placing(
        &mut self,
        environment: &mut Environment,
        item_vault: &ItemVault,
        pos: Point<Float>,
    ) {
        self.about_to_delete_selected = false;
        match &self.state {
            EditorState::Selection { .. } => panic!("Cannot end placing when in selection mode"),
            EditorState::Placing { preset_to_place } => {
                environment.add_vessel(preset_to_place.create_vessel(item_vault, pos));
                self.state = EditorState::Selection {
                    selected_vessel_id: None,
                };
            }
        }
    }
}
