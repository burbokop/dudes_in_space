use crate::camera::Camera;
use crate::render::{FontProvider, RenderError};
use dudes_in_space_api::vessel::Vessel;

pub struct PersonRenderModel {}

impl PersonRenderModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        texture_creator: &mut sdl2::render::TextureCreator<T::Context>,
        font_provider: &FontProvider,
        camera: &Camera,
        vessel: &Vessel,
    ) -> Result<(), RenderError> {
        let tr = camera.transformation();

        Ok(())
    }
}
