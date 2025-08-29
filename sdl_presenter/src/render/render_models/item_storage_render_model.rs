use std::fmt::Alignment;
use dudes_in_space_api::item::ItemStorage;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;
use crate::render::{RenderError, Renderer};
use crate::render::scene_graph::{GraphicsNode, GridLayout};

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
        let layout : GridLayout<_> =
            storage.content().map(|stack|{
                |renderer: &mut Renderer<T>, bounding_box| {
                    let stack = stack.clone();
                    renderer.draw_confined_text(
                        &format!("{}\n{}", stack.id(), stack.count()),
                        bounding_box,
                        Alignment::Center,
                        Color::black(),
                    );
                    renderer.draw_rect(bounding_box, Color::black());
                }
            }).collect();

        layout.draw(renderer, bounding_box);
        renderer.draw_rect(bounding_box, Color::black());
        
        Ok(())
    }
}
