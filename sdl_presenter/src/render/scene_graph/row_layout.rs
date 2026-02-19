use crate::render::scene_graph::GraphicsNode;
use crate::render::{OLD_DEFAULT_MARGIN, Renderer};
use burbomath::math::{Rect, Size};
use dudes_in_space_api::utils::utils::Float;

pub struct RowLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn GraphicsNode<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> From<RowLayout<'a, T>>
    for Box<dyn GraphicsNode<T> + 'a>
{
    fn from(value: RowLayout<'a, T>) -> Self {
        Box::new(value)
    }
}

impl<'a, T: sdl2::render::RenderTarget + 'a> RowLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Self {
        Self { elems }
    }

    pub fn boxed(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self { elems })
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for RowLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().any(|x| x.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) =
            bounding_box.homogeneous_mul(OLD_DEFAULT_MARGIN.assume_relative().unwrap().value());

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
    fn implicit_size(&self) -> Option<Size<Float>> {
        let v: Vec<_> = self
            .elems
            .iter()
            .filter_map(|elem| elem.visible().then(|| elem.implicit_size()).flatten())
            .collect();

        if v.is_empty() {
            return None;
        }

        let mut result: Size<_> = (0., 0.).into();

        for size in v {
            result = (*result.w() + *size.w(), Float::max(*result.h(), *size.h())).into();
        }

        Some(result)
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
    elems: Vec<(ExtRowLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> ExtRowLayout<'a, T> {
    pub(crate) fn new(elems: Vec<(ExtRowLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>) -> Self {
        Self { elems }
    }

    pub fn boxed(
        elems: Vec<(ExtRowLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
    ) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self { elems })
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ExtRowLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().any(|(_, x)| x.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        // TODO: allow more than 2 elements
        assert_eq!(self.elems.len(), 2);

        let (_, elem0) = &self.elems[0];
        let (_, elem1) = &self.elems[1];

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

    fn implicit_size(&self) -> Option<Size<Float>> {
        let v: Vec<_> = self
            .elems
            .iter()
            .filter_map(|(_, elem)| elem.visible().then(|| elem.implicit_size()).flatten())
            .collect();

        if v.is_empty() {
            return None;
        }

        let mut result: Size<_> = (0., 0.).into();

        for size in v {
            result = (*result.w() + *size.w(), Float::max(*result.h(), *size.h())).into();
        }

        Some(result)
    }
}
