use crate::render::FontProvider;
use dudes_in_space_api::utils::math::{Matrix, Rect};
use dudes_in_space_api::utils::utils::Float;
use sdl2::render::{Canvas, TextureCreator};

pub trait LayoutElement<T: sdl2::render::RenderTarget> {
    fn visible(&self) -> bool;
    fn draw(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        texture_creator: &sdl2::render::TextureCreator<T::Context>,
        font_provider: &FontProvider,
        tr: &Matrix<Float>,
        view_port_in_world_space: Rect<Float>,
        bounding_box: Rect<Float>,
    );
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

    fn draw(
        &self,
        canvas: &mut Canvas<T>,
        texture_creator: &TextureCreator<T::Context>,
        font_provider: &FontProvider,
        tr: &Matrix<Float>,
        view_port_in_world_space: Rect<Float>,
        bounding_box: Rect<Float>,
    ) {
        if !view_port_in_world_space.instersects(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|x| x.visible()).count();
        if count == 0 {
            return;
        }

        let x = *bounding_box.x();
        let y = *bounding_box.y();
        let w = *bounding_box.w();
        let h = bounding_box.h() / count as Float;

        let mut i = 0;

        for elem in &self.elems {
            if elem.visible() {
                elem.draw(
                    canvas,
                    texture_creator,
                    font_provider,
                    tr,
                    view_port_in_world_space,
                    (x, i as Float * y, w, h).into(),
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

    fn draw(
        &self,
        canvas: &mut Canvas<T>,
        texture_creator: &TextureCreator<T::Context>,
        font_provider: &FontProvider,
        tr: &Matrix<Float>,
        view_port_in_world_space: Rect<Float>,
        bounding_box: Rect<Float>,
    ) {
        if !view_port_in_world_space.instersects(&bounding_box) {
            return;
        }

        let count = self.elems.iter().filter(|x| x.visible()).count();
        if count == 0 {
            return;
        }

        let x = *bounding_box.x();
        let y = *bounding_box.y();
        let w = bounding_box.w() / count as Float;
        let h = *bounding_box.h();

        let mut i = 0;

        for elem in &self.elems {
            if elem.visible() {
                elem.draw(
                    canvas,
                    texture_creator,
                    font_provider,
                    tr,
                    view_port_in_world_space,
                    (i as Float * x, y, w, h).into(),
                );
                i += 1;
            }
        }
    }
}
