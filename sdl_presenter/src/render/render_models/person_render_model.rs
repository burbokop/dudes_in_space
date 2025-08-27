use crate::render::{RenderError, Renderer};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Point, Rect};
use dudes_in_space_api::utils::utils::Float;
use std::convert::Into;
use std::ops::Deref;
use std::sync::LazyLock;

fn draw_little_man<T: sdl2::render::RenderTarget>(
    renderer: &mut Renderer<T>,
    bounding_box: Rect<Float>,
) {
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

    let aabb: Rect<Float> = Rect::aabb_from_points(POINTS.deref().iter().cloned()).unwrap();

    let qx = bounding_box.w() / aabb.w();
    let qy = bounding_box.h() / aabb.h();

    let q = qx.min(qy);

    let points: Vec<_> = POINTS
        .iter()
        .map(|x| bounding_box.left_top() + (*x - aabb.left_top()) * q)
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
        draw_little_man(renderer, bounding_box);
        Ok(())
    }
}
