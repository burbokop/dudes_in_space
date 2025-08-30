use crate::render::scene_graph::GraphicsNode;
use crate::render::{DEFAULT_MARGIN, Renderer};
use dudes_in_space_api::utils::math::{Rect, Size};
use dudes_in_space_api::utils::utils::Float;

pub struct RowLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn GraphicsNode<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget> RowLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for RowLayout<'a, T> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) = bounding_box.homogeneous_mul(DEFAULT_MARGIN);

        if !renderer.intersects_with_view_port(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|x| x.visible()).count();
        if count == 0 {
            return;
        }

        let margin = margin / 2.;
        let sum_margin = margin.abs() * (count - 1) as Float;

        let x = *bounding_box.x();
        let y = *bounding_box.y();
        let w = (bounding_box.w() - sum_margin) / count as Float;
        let h = *bounding_box.h();

        let mut i = 0;

        for elem in &self.elems {
            if elem.visible() {
                elem.draw(
                    renderer,
                    (i as Float * (w + margin.abs()) + x, y, w, h).into(),
                );
                i += 1;
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct ExtRowLayoutOptions {
    preserve_aspect_ratio: bool,
    relative_width: Option<Float>,
}

impl ExtRowLayoutOptions {
    pub fn preserve_aspect_ratio() -> Self {
        Self {
            preserve_aspect_ratio: true,
            relative_width: None,
        }
    }
}

pub struct ExtRowLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<(Box<dyn GraphicsNode<T> + 'a>, ExtRowLayoutOptions)>,
}

impl<'a, T: sdl2::render::RenderTarget> ExtRowLayout<'a, T> {
    pub(crate) fn new(elems: Vec<(Box<dyn GraphicsNode<T> + 'a>, ExtRowLayoutOptions)>) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ExtRowLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().all(|x| x.0.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        // TODO: allow more than 2 elements
        assert_eq!(self.elems.len(), 2);

        let (elem0, _) = &self.elems[0];
        let (elem1, _) = &self.elems[1];

        let e0_size = elem0.implicit_size().unwrap();

        let qx = bounding_box.w() / e0_size.w();
        let qy = bounding_box.h() / e0_size.h();

        let q = qx.min(qy);

        let e0_new_size: Size<Float> =
            ((qy * e0_size.w()).min(*bounding_box.w()), *bounding_box.h()).into();

        let e0_bb: Rect<_> = (bounding_box.left_top(), e0_new_size).into();

        let e1_bb = Rect::from_lrtb_unchecked(
            e0_bb.right(),
            bounding_box.right(),
            bounding_box.top(),
            bounding_box.bottom(),
        );

        elem0.draw(renderer, e0_bb);
        elem1.draw(renderer, e1_bb);
    }
}
