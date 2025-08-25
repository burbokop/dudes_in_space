use crate::render::DEFAULT_MARGIN;
use crate::render::renderer::Renderer;
use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;

pub trait LayoutElement<T: sdl2::render::RenderTarget> {
    fn visible(&self) -> bool;
    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>);
}

pub struct ColumnLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn LayoutElement<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget> ColumnLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn LayoutElement<T> + 'a>>) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for ColumnLayout<'a, T> {
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

pub struct RowLayout<'a, T: sdl2::render::RenderTarget> {
    elems: Vec<Box<dyn LayoutElement<T> + 'a>>,
}

impl<'a, T: sdl2::render::RenderTarget> RowLayout<'a, T> {
    pub fn new(elems: Vec<Box<dyn LayoutElement<T> + 'a>>) -> Self {
        Self { elems }
    }
}

impl<'a, T: sdl2::render::RenderTarget> LayoutElement<T> for RowLayout<'a, T> {
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
