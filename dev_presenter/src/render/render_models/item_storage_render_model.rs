use crate::render::scene_graph::{GraphicsNode, GridLayout};
use crate::render::{Alignment, RenderError, Renderer};
use burbomath::Rect;
use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::utils::Float;

pub struct ItemStorageRenderModel {}

impl ItemStorageRenderModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        storage: &ItemStorage,
        bounding_box: Rect<Float>,
    ) -> Result<(), RenderError> {
        let layout: GridLayout<_> = storage
            .content()
            .stacks()
            .map(|stack| {
                |renderer: &mut Renderer<T>, bounding_box| {
                    let stack = stack.clone();
                    renderer.draw_confined_text(
                        &format!("{}\n{}", stack.id(), stack.count()),
                        bounding_box,
                        Alignment::center(),
                        Color::black(),
                    );
                    renderer.draw_rect(bounding_box, Color::black());
                }
            })
            .collect();

        layout.draw(renderer, bounding_box);
        renderer.draw_rect(bounding_box, Color::black());

        Ok(())
    }
}
