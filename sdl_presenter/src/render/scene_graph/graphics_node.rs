use crate::render::Renderer;
use burbomath::math::{Rect, Size};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::utils::Float;
use std::ops::Add;

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

impl<'a, T: sdl2::render::RenderTarget + 'a> Add<Box<dyn GraphicsNode<T> + 'a>>
    for Box<dyn GraphicsNode<T> + 'a>
{
    type Output = Box<dyn GraphicsNode<T> + 'a>;

    fn add(self, rhs: Box<dyn GraphicsNode<T> + 'a>) -> Self::Output {
        struct SumGraphicsNode<'a, 'b, T: sdl2::render::RenderTarget> {
            bg: Box<dyn GraphicsNode<T> + 'a>,
            fg: Box<dyn GraphicsNode<T> + 'b>,
        }

        impl<'a, 'b, T: sdl2::render::RenderTarget> GraphicsNode<T> for SumGraphicsNode<'a, 'b, T> {
            fn visible(&self) -> bool {
                self.bg.visible() || self.fg.visible()
            }

            fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
                self.bg.draw(renderer, bounding_box);
                self.fg.draw(renderer, bounding_box);
            }
        }

        Box::new(SumGraphicsNode::<'a, 'a, T> { bg: self, fg: rhs })
    }
}

pub struct Frame;

impl Frame {
    pub fn boxed<'a, T: sdl2::render::RenderTarget + 'a>() -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self)
    }
}

impl<'a, T: sdl2::render::RenderTarget + 'a> From<Frame> for Box<dyn GraphicsNode<T> + 'a> {
    fn from(value: Frame) -> Self {
        Box::new(value)
    }
}

impl<'a, T: sdl2::render::RenderTarget + 'a> GraphicsNode<T> for Frame {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        renderer.draw_rect(bounding_box, Color::black())
    }
}
