use crate::camera::Camera;
use crate::render::draw_utils::DrawConfinedText;
use crate::render::font_provider::FontProvider;
use crate::render::render_models::person_render_model::PersonRenderModel;
use crate::render::{RenderError, rect_to_sdl2_rect};
use dudes_in_space_api::module::Module;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Matrix, Rect};
use dudes_in_space_api::utils::utils::Float;

fn draw_top_info<T: 
sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    texture_creator: &sdl2::render::TextureCreator<T::Context>,
    font_provider: &FontProvider,
    tr: &Matrix<Float>,
    module: &dyn Module,
    bounding_box: Rect<Float>,
) {
    let text_box_height = 0.5;

    let text_box: Rect<Float> = (
        *bounding_box.x(),
        bounding_box.y() - text_box_height,
        *bounding_box.w(),
        text_box_height,
    )
        .into();

    canvas.draw_confined_text(
        texture_creator,
        font_provider,
        &format!("{} ({}:{})", module.id(), module.package_id(), module.type_id()),
        tr * &text_box,
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
}


fn draw_bottom_info<T:
sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    texture_creator: &sdl2::render::TextureCreator<T::Context>,
    font_provider: &FontProvider,
    tr: &Matrix<Float>,
    module: &dyn Module,
    bounding_box: Rect<Float>,
) {
    let text_box_height = 0.5;

    let text_box: Rect<Float> = (
        *bounding_box.x(),
        bounding_box.bottom(),
        *bounding_box.w(),
        text_box_height,
    )
        .into();
    
    canvas.draw_confined_text(
        texture_creator,
        font_provider,
        &format!("{:?} <=> {:?}", module.capabilities(), module.primary_capabilities()),
        tr * &text_box,
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
}

fn draw_bounding_box<T:
sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    tr: &Matrix<Float>,
    bounding_box: Rect<Float>,
) {
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas
        .draw_rect(rect_to_sdl2_rect(tr * &bounding_box))
        .unwrap();
}

pub struct ModuleRenderModel {
    person_render_model: PersonRenderModel,
}

impl ModuleRenderModel {
    pub fn new() -> Self {
        Self {
            person_render_model: PersonRenderModel::new(),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        texture_creator: &mut sdl2::render::TextureCreator<T::Context>,
        font_provider: &FontProvider,
        camera: &Camera,
        module: &dyn Module,
        bounding_box: Rect<Float>,
    ) -> Result<(), RenderError> {
        let tr = camera.transformation();

        draw_top_info(canvas,texture_creator,  font_provider, &tr, module, bounding_box);
        draw_bottom_info(canvas, texture_creator, font_provider, &tr, module, bounding_box);
        draw_bounding_box(canvas, &tr, bounding_box);
        
        // let _ = module.item_recipes();
        // let _ = module.input_item_recipes();
        // let _ = module.output_item_recipes();
        // let _ = module.assembly_recipes();
        // let _ = module.free_person_slots_count();
        // let _ = module.persons();
        // let _ = module.storages();
        // let _ = module.safes();
        // let _ = module.module_storages();
        // let _ = module.docking_clamps();
        // let _ = module.docking_connectors();
        // let _ = module.trading_console();

        Ok(())
    }
}
