use crate::camera::Camera;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use dudes_in_space_api::environment::{Environment, EnvironmentSeed};

pub struct RenderModel {}

impl RenderModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        environment: &Environment,
        camera: &Camera,
    ) -> Result<(), RenderError> {
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(())
    }
}

#[derive(Debug)]
pub enum RenderError {}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for RenderError {}
