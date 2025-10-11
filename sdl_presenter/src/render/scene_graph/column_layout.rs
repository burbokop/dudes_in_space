use crate::render::scene_graph::GraphicsNode;
use crate::render::{DEFAULT_MARGIN, Margins, Renderer};
use dudes_in_space_api::utils::math::{Rect, Size};
use dudes_in_space_api::utils::utils::Float;

pub struct ColumnLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn GraphicsNode<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> ColumnLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Self {
        Self { elems }
    }

    pub fn boxed(elems: Vec<Box<dyn GraphicsNode<T> + 'a>>) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self { elems })
    }
}

impl<'a, T: sdl2::render::RenderTarget + 'a> From<ColumnLayout<'a, T>>
    for Box<dyn GraphicsNode<T> + 'a>
{
    fn from(value: ColumnLayout<'a, T>) -> Self {
        Box::new(value)
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ColumnLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().any(|x| x.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) =
            bounding_box.homogeneous_mul(DEFAULT_MARGIN.assume_relative().unwrap().value());

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
            result = (Float::max(*result.w(), *size.w()), *result.h() + *size.h()).into();
        }

        Some(result)
    }
}

#[derive(Default)]
pub(crate) struct ExtColumnLayoutOptions {
    preserve_aspect_ratio: bool,
    relative_height: Option<Float>,
    margins: Margins,
}

impl ExtColumnLayoutOptions {
    pub fn relative_height(h: Float) -> Self {
        Self {
            preserve_aspect_ratio: false,
            relative_height: Some(h),
            margins: Default::default(),
        }
    }

    pub fn margins(margins: Margins) -> Self {
        Self {
            preserve_aspect_ratio: false,
            relative_height: None,
            margins,
        }
    }
}

pub struct ExtColumnLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<(ExtColumnLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> ExtColumnLayout<'a, T> {
    pub(crate) fn new(elems: Vec<(ExtColumnLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>) -> Self {
        Self { elems }
    }

    pub fn boxed(
        elems: Vec<(ExtColumnLayoutOptions, Box<dyn GraphicsNode<T> + 'a>)>,
    ) -> Box<dyn GraphicsNode<T> + 'a> {
        Box::new(Self { elems })
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for ExtColumnLayout<'a, T> {
    fn visible(&self) -> bool {
        self.elems.iter().any(|(_, x)| x.visible())
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let (bounding_box, margin) =
            bounding_box.homogeneous_mul(DEFAULT_MARGIN.assume_relative().unwrap().value());

        if !renderer.intersects_with_view_port(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|(_, x)| x.visible()).count();
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
            .filter(|(options, elem)| elem.visible() && options.relative_height.is_some())
            .count();
        let sum_relative_height = self
            .elems
            .iter()
            .filter_map(|(options, elem)| {
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

        for (options, elem) in &self.elems {
            if elem.visible() {
                let h = options.relative_height.unwrap_or(rest_relative_height) * bounding_box.h();

                elem.draw(renderer, (x, y, w, h).into());
                y += h;
            }
        }
    }

    fn implicit_size(&self) -> Option<Size<Float>> {
        let v: Vec<_> = self
            .elems
            .iter()
            .filter_map(|(options, elem)| elem.visible().then(|| elem.implicit_size()).flatten())
            .collect();

        if v.is_empty() {
            return None;
        }

        let mut result: Size<_> = (0., 0.).into();

        for size in v {
            result = (Float::max(*result.w(), *size.w()), *result.h() + *size.h()).into();
        }

        Some(result)
    }
}
