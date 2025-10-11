use crate::render::scene_graph::GraphicsNode;
use crate::render::{Alignment, Pix, Renderer};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Rect, Size};
use dudes_in_space_api::utils::utils::Float;

impl<T: sdl2::render::RenderTarget> GraphicsNode<T> for String {
    fn visible(&self) -> bool {
        !self.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(&self, bounding_box, Alignment::center(), Color::black());
    }
}

impl<T: sdl2::render::RenderTarget> From<String> for Box<dyn GraphicsNode<T>> {
    fn from(value: String) -> Self {
        Box::new(value)
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for &'a str {
    fn visible(&self) -> bool {
        !self.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(&self, bounding_box, Alignment::center(), Color::black());
    }
}

pub(crate) struct Text {
    pub text: String,
    pub color: Color,
    pub alignment: Alignment,
    pub font_height: Option<Pix>,
}

impl<T: sdl2::render::RenderTarget> From<Text> for Box<dyn GraphicsNode<T>> {
    fn from(value: Text) -> Self {
        Box::new(value)
    }
}

impl<T: sdl2::render::RenderTarget> GraphicsNode<T> for Text {
    fn visible(&self) -> bool {
        !self.text.is_empty()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_confined_text(&self.text, bounding_box, self.alignment, self.color.clone());
    }
    fn implicit_size(&self) -> Option<Size<Float>> {
        self.font_height.map(|x| {
            let s = Renderer::<T>::text_size(&self.text);
            (x.0 * *s.w() as Float, x.0 * *s.h() as Float).into()
        })
    }
}
