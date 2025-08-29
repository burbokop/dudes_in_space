use crate::render::scene_graph::GraphicsNode;
use crate::render::{HorisontalAlignment, Renderer};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;

impl<T: sdl2::render::RenderTarget> GraphicsNode<T> for String {
    fn visible(&self) -> bool {
        !self.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(
            &self,
            bounding_box,
            HorisontalAlignment::Center,
            Color::black(),
        );
    }
}

pub struct Text {
    pub text: String,
    pub color: Color,
    pub alignment: HorisontalAlignment,
}

impl<T: sdl2::render::RenderTarget> GraphicsNode<T> for Text {
    fn visible(&self) -> bool {
        !self.text.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(&self.text, bounding_box, self.alignment, self.color.clone());
    }
}
