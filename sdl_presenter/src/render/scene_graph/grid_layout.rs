use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;
use crate::render::{Renderer, DEFAULT_MARGIN};
use crate::render::scene_graph::{ GraphicsNode};

pub struct GridLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn GraphicsNode<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget> GridLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget, N: GraphicsNode<T>+'a>  FromIterator<N> for GridLayout<'a, T > {
    fn from_iter<I: IntoIterator<Item=N>>(iter: I) -> Self {
        Self { elems: iter.into_iter().map(|e| Box::new(e) as Box<dyn GraphicsNode<T>>).collect() }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for GridLayout<'a, T> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, delta) = bounding_box.homogeneous_mul(DEFAULT_MARGIN);
        let margin = delta.abs() / 2.;
        let cell_spacing = delta.abs() / 2.;

        let n = self.elems.len();
        let w = bounding_box.w();
        let h = bounding_box.h();

        let q = w / h;

        let cy = (n as Float / q).sqrt().ceil() as usize;
        let cx = (n as Float * q).sqrt().ceil() as usize;

        let cell_width = (h / cy as Float).min(w / cx as Float);

        let mut i = 0;
        for x in 0..cx {
            for y in 0..cy {
                if i >= n {
                    break;
                }

                let bounding_box = (
                    (
                        x as Float * (cell_width + cell_spacing) + bounding_box.x(),
                        y as Float * (cell_width + cell_spacing) + bounding_box.y(),
                    )
                        .into(),
                    (cell_width, cell_width).into(),
                )
                    .into();

                self.elems[i].draw(renderer, bounding_box);
                i += 1;
            }
        }





    }
}
