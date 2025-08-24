use crate::camera::Camera;
use crate::render::font_provider::FontProvider;
use crate::render::{RenderError, VesselRenderModel};
use dudes_in_space_api::environment::Environment;

pub struct EnvironmentRenderModel {
    vessel_render_model: VesselRenderModel,
}

impl EnvironmentRenderModel {
    pub fn new() -> Self {
        Self {
            vessel_render_model: VesselRenderModel::new(),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        texture_creator: &mut sdl2::render::TextureCreator<T::Context>,
        font_provider: &FontProvider,
        camera: &Camera,
        environment: &Environment,
    ) -> Result<(), RenderError> {
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        canvas.clear();

        let tr = camera.transformation();

        for vessel in environment.vessels() {
            self.vessel_render_model
                .render(canvas, texture_creator, font_provider, camera, vessel)?;
        }

        Ok(())
    }
}
