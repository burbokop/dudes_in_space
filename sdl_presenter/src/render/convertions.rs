use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Point, Rect};
use dudes_in_space_api::utils::utils::Float;

pub fn color_to_sdl2_rgba_color(c: Color) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(
        (c.r * 255.) as u8,
        (c.g * 255.) as u8,
        (c.b * 255.) as u8,
        (c.a * 255.) as u8,
    )
}

pub fn rect_to_sdl2_rect(c: Rect<Float>) -> sdl2::rect::Rect {
    (*c.x() as i32, *c.y() as i32, *c.w() as u32, *c.h() as u32).into()
}

pub fn point_to_sdl2_point(c: Point<Float>) -> sdl2::rect::Point {
    (*c.x() as i32, *c.y() as i32).into()
}
