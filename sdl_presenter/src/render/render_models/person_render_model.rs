use crate::render::{RenderError, Renderer};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Point, Rect};
use dudes_in_space_api::utils::utils::Float;
use std::convert::Into;
use std::ops::Deref;
use std::sync::LazyLock;

static POINTS: LazyLock<[Point<Float>; 16]> = LazyLock::new(|| {
    [
        (-1., 14.).into(),
        (-3., 13.).into(),
        (-4., 10.).into(),
        (-2., 12.).into(),
        (-2., 8.).into(),
        (-3., 0.).into(),
        (0., 6.).into(),
        (3., 0.).into(),
        (2., 8.).into(),
        (2., 12.).into(),
        (4., 10.).into(),
        (3., 13.).into(),
        (1., 14.).into(),
        (2., 17.).into(),
        (0., 18.).into(),
        (-2., 17.).into(),
    ]
});

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
        let bb: Rect<Float> = Rect::aabb_from_points(POINTS.deref().iter().cloned()).unwrap();

        let x = bounding_box.w() / bb.w();
        let y = bounding_box.h() / bb.h();

        let q = x.min(y);

        let ppp: Vec<_> = POINTS
            .iter()
            .map(|x| bounding_box.left_top() + (*x - Point::origin()) * q)
            .collect();

        renderer.draw_polygon(&ppp, Color::black());

        Ok(())
    }
}
