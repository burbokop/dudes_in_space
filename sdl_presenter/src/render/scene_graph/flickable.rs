use crate::render::Renderer;
use crate::render::scene_graph::GraphicsNode;
use dudes_in_space_api::utils::math::{Rect, Vector};
use dudes_in_space_api::utils::utils::Float;

pub(crate) enum FlickableDirection {
    Horizontal,
    Vertical,
}

pub(crate) struct Flickable<'a, T: sdl2::render::RenderTarget> {
    direction: FlickableDirection,
    content_offset: Vector<Float>,
    content: Box<dyn GraphicsNode<T> + 'a>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> Flickable<'a, T> {
    pub fn new(
        direction: FlickableDirection,
        content_offset: Vector<Float>,
        content: Box<dyn GraphicsNode<T> + 'a>,
    ) -> Self {
        Self {
            direction,
            content_offset,
            content,
        }
    }

    pub fn boxed(
        direction: FlickableDirection,
        content_offset: Vector<Float>,
        content: Box<dyn GraphicsNode<T> + 'a>,
    ) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self {
            direction,
            content_offset,
            content,
        })
    }
}

impl<'a, T: sdl2::render::RenderTarget + 'a> From<Flickable<'a, T>>
    for Box<dyn GraphicsNode<T> + 'a>
{
    fn from(value: Flickable<'a, T>) -> Self {
        Box::new(value)
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for Flickable<'a, T> {
    fn visible(&self) -> bool {
        self.content.visible()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        match self.direction {
            FlickableDirection::Horizontal => todo!(),
            FlickableDirection::Vertical => {
                let content_bb = (
                    *bounding_box.x() + *self.content_offset.x(),
                    *bounding_box.y() + *self.content_offset.y(),
                    *bounding_box.w(),
                    bounding_box
                        .h()
                        .max(*self.content.implicit_size().unwrap().h()),
                )
                    .into();

                self.content.draw(renderer, content_bb);
            }
        }
    }
}
