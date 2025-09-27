use crate::logger::MemLogger;
use crate::person_table::PersonTable;
use crate::render::renderer::Renderer;
use crate::render::{ModuleTextureContainerRef, RenderError, VesselRenderModel};
use dudes_in_space_api::environment::Environment;

pub struct EnvironmentRenderModel<'texture> {
    vessel_render_model: VesselRenderModel<'texture>,
}

impl<'texture> EnvironmentRenderModel<'texture> {
    pub fn new(backgrounds: ModuleTextureContainerRef<'texture>) -> Self {
        Self {
            vessel_render_model: VesselRenderModel::new(backgrounds),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        environment: &Environment,
        logger: &MemLogger,
        person_table: &PersonTable,
    ) -> Result<(), RenderError> {
        for vessel in environment.vessels() {
            self.vessel_render_model
                .render(renderer, vessel, logger, person_table, None)?;
        }
        Ok(())
    }
}
