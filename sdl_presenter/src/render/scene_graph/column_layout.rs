use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;
use crate::render::{Renderer, DEFAULT_MARGIN};
use crate::render::scene_graph::GraphicsNode;

pub struct ColumnLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn GraphicsNode<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget> ColumnLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ColumnLayout<'a, T> {
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
        let w = *bounding_box.w();
        let h = (bounding_box.h() - sum_margin) / count as Float;

        let mut i = 0;

        for elem in &self.elems {
            if elem.visible() {
                elem.draw(
                    renderer,
                    (x, i as Float * (h + margin.abs()) + y, w, h).into(),
                );
                i += 1;
            }
        }
    }
}



#[derive(Default)]
pub(crate) struct ExtColumnLayoutOptions {
    preserve_aspect_ratio: bool,
    relative_height: Option<Float>,
}

impl ExtColumnLayoutOptions {
    pub fn relative_height(h: Float) -> Self {
        Self {
            preserve_aspect_ratio: false,
            relative_height: Some(h),
        }
    }
}

pub struct ExtColumnLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<(Box<dyn GraphicsNode<T> + 'a>, ExtColumnLayoutOptions)>,
}

impl<'a, T: sdl2::render::RenderTarget> ExtColumnLayout<'a, T> {
    pub(crate) fn new(
        elems: Vec<(Box<dyn GraphicsNode<T> + 'a>, ExtColumnLayoutOptions)>,
    ) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ExtColumnLayout<'a, T> {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) = bounding_box.homogeneous_mul(DEFAULT_MARGIN);

        if !renderer.intersects_with_view_port(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|(x, _)| x.visible()).count();
        if count == 0 {
            return;
        }

        let margin = margin / 2.;
        let sum_margin = margin.abs() * (count - 1) as Float;

        let x = *bounding_box.x();
        let mut y = *bounding_box.y();
        let w = *bounding_box.w();

        let elems_with_relative_height_count = self
            .elems
            .iter()
            .filter(|(elem, options)| elem.visible() && options.relative_height.is_some())
            .count();
        let sum_relative_height = self
            .elems
            .iter()
            .filter_map(|(elem, options)| {
                if elem.visible() {
                    options.relative_height
                } else {
                    None
                }
            })
            .sum::<Float>();
        assert!(sum_relative_height >= 0.);
        assert!(sum_relative_height <= 1.);

        let rest_relative_height = (1. - sum_relative_height)
            / (self.elems.len() - elems_with_relative_height_count) as Float;

        for (elem, options) in &self.elems {
            if elem.visible() {
                let h = options.relative_height.unwrap_or(rest_relative_height) * bounding_box.h();

                elem.draw(renderer, (x, y, w, h).into());
                y += h;
            }
        }
    }
}