use crate::render::RenderError;
use crate::render::render_models::module_render_model::ModuleRenderModel;
use crate::render::renderer::Renderer;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Rect, Vector};
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_api::vessel::Vessel;
use std::ops::Deref;

pub struct VesselRenderModel {
    module_render_model: ModuleRenderModel,
}

impl VesselRenderModel {
    pub fn new() -> Self {
        Self {
            module_render_model: ModuleRenderModel::new(),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        vessel: &Vessel,
    ) -> Result<(), RenderError> {
        let modules: Vec<_> = vessel.modules().collect();
        let modules_count = modules.len();
        let side_count = (modules_count as Float).sqrt().ceil() as usize;

        if side_count == 0 {
            return Ok(());
        }

        let cell_width = 10.;
        let cell_spacing = 1.;
        let margin = 2.;

        let side_height =
            cell_width * side_count as Float + cell_spacing * (side_count - 1) as Float;

        let side_width = cell_width * (modules_count as Float / side_count as Float).ceil()
            + cell_spacing * ((modules_count as Float / side_count as Float).ceil() - 1.);

        let rect = Rect::from_center(
            vessel.pos() - Vector::from((margin, margin)),
            (side_width + margin * 2., side_height + margin * 2.).into(),
        );

        renderer.draw_rect(
            rect,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        );

        let mut i = 0;
        for x in 0..side_count {
            for y in 0..side_count {
                if i >= modules_count {
                    break;
                }

                let bounding_box = (
                    (
                        x as Float * (cell_width + cell_spacing) + rect.x() + margin,
                        y as Float * (cell_width + cell_spacing) + rect.y() + margin,
                    )
                        .into(),
                    (cell_width, cell_width).into(),
                )
                    .into();

                self.module_render_model
                    .render(renderer, modules[i].deref(), bounding_box)?;
                i += 1;
            }
        }

        Ok(())
    }
}
