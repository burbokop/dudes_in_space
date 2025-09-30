use dudes_in_space_api::utils::math::Point;
use dudes_in_space_api::utils::utils::Float;

pub(crate) enum EditorState {
    Selection,
    Placing { preset_to_place: EditorPreset },
}

#[derive(Debug)]
pub(crate) enum EditorPreset {
    Preset0,
}

pub(crate) struct Editor {
    state: EditorState,
}
impl Editor {
    pub(crate) fn new() -> Self {
        Self {
            state: EditorState::Selection,
        }
    }

    pub(crate) fn state(&self) -> &EditorState {
        &self.state
    }

    pub(crate) fn begin_placing(&mut self, preset: EditorPreset) {
        self.state = EditorState::Placing {
            preset_to_place: preset,
        }
    }

    pub(crate) fn end_placing(&mut self, pos: Point<Float>) {
        todo!()
    }
}
