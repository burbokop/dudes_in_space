use crate::render::Renderer;
use dudes_in_space_api::utils::math::{Rect, Size};
use dudes_in_space_api::utils::utils::Float;

pub trait GraphicsNode<T: sdl2::render::RenderTarget> {
    fn visible(&self) -> bool;
    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>);
    fn implicit_size(&self) -> Option<Size<Float>> {
        None
    }
}

impl<T: sdl2::render::RenderTarget, F: Fn(&mut Renderer<T>, Rect<Float>) -> ()> GraphicsNode<T>
    for F
{
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        self.call((renderer, bounding_box))
    }
}
