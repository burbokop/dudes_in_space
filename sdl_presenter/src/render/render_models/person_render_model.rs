use crate::render::{ExtColumnLayout, ExtColumnLayoutOptions, ExtRowLayout, ExtRowLayoutOptions, LayoutElement, RenderError, Renderer};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Point, Rect, Size};
use dudes_in_space_api::utils::utils::Float;
use std::convert::Into;
use std::ops::Deref;
use std::sync::LazyLock;

struct DrawLittleMan {
    points: &'static [Point<Float>],
    aabb: Rect<Float>
}

impl DrawLittleMan {
    fn new() -> Self {
        static POINTS: LazyLock<[Point<Float>; 16]> = LazyLock::new(|| {
            [
                (0., 0.).into(),
                (-2., 1.).into(),
                (-1., 4.).into(),
                (-3., 5.).into(),
                (-4., 8.).into(),
                (-2., 6.).into(),
                (-2., 10.).into(),
                (-3., 18.).into(),
                (0., 12.).into(),
                (3., 18.).into(),
                (2., 10.).into(),
                (2., 6.).into(),
                (4., 8.).into(),
                (3., 5.).into(),
                (1., 4.).into(),
                (2., 1.).into(),
            ]
        });
        Self {
            points: POINTS.deref(),
            aabb:Rect::aabb_from_points(POINTS.deref().iter().cloned()).unwrap(),
        }
    }
}

impl<T: sdl2::render::RenderTarget> LayoutElement<T> for DrawLittleMan {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {

        let qx = bounding_box.w() / self.aabb.w();
        let qy = bounding_box.h() / self.aabb.h();

        let q = qx.min(qy);

        let points: Vec<_> = self.points
            .iter()
            .map(|x| bounding_box.left_top() + (*x - self.aabb.left_top()) * q)
            .collect();

        renderer.draw_polygon(&points, Color::black());

        for p in points {
            renderer.fill_circle(
                p,
                bounding_box.w().min(*bounding_box.h()) / 50.,
                Color::black(),
            );
        }
    }

    fn implicit_size(&self) -> Option<Size<Float>> {
        todo!()
    }
}

struct DrawLog {
    
}

impl DrawLog {
    fn new() -> Self {
        Self {}
    }
}

impl<T: sdl2::render::RenderTarget> LayoutElement<T> for DrawLog {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}






struct DrawHeader {

}

impl DrawHeader {
    fn new() -> Self {
        Self {}
    }
}

impl<T: sdl2::render::RenderTarget> LayoutElement<T> for DrawHeader {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}






struct DrawFooter {

}

impl DrawFooter {
    fn new() -> Self {
        Self {}
    }
}

impl<T: sdl2::render::RenderTarget> LayoutElement<T> for DrawFooter {
    fn visible(&self) -> bool {
        todo!()
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        todo!()
    }
}




pub struct PersonRenderModel {}

impl PersonRenderModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        person: &Person,
        bounding_box: Rect<Float>,
    ) -> Result<(), RenderError> {
        
        let row = ExtRowLayout::new(vec![
            (Box::new(DrawLittleMan::new()), ExtRowLayoutOptions::preserve_aspect_ratio()),
            (Box::new(DrawLog::new()), Default::default()),
        ]);
        
        let column = ExtColumnLayout::new(vec![
            (Box::new(DrawFooter::new()), Default::default()),
            (Box::new(row), ExtColumnLayoutOptions::relative_height(0.5)),
            (Box::new(DrawHeader::new()), Default::default()),
        ]);

        column.draw(renderer, bounding_box);
        
        Ok(())
    }
}
