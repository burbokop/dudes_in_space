use crate::render::font_provider::FontProvider;
use crate::render::{color_to_sdl2_rgba_color, point_to_sdl2_point};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Point, Rect};
use dudes_in_space_api::utils::utils::Float;

pub trait DrawCenteredText {
    type Context;
    fn draw_centered_text(
        &mut self,
        texture_creator: &sdl2::render::TextureCreator<Self::Context>,
        font: &sdl2::ttf::Font,
        text: &str,
        center: Point<Float>,
        color: Color,
    );
}

impl<T> DrawCenteredText for sdl2::render::Canvas<T>
where
    T: sdl2::render::RenderTarget,
{
    type Context = T::Context;

    fn draw_centered_text(
        &mut self,
        texture_creator: &sdl2::render::TextureCreator<T::Context>,
        font: &sdl2::ttf::Font,
        text: &str,
        center: Point<Float>,
        color: Color,
    ) {
        if text.len() > 0 {
            let surface = font
                .render(text)
                .blended(color_to_sdl2_rgba_color(color))
                .map_err(|e| e.to_string())
                .unwrap();

            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            let sdl2::render::TextureQuery { width, height, .. } = texture.query();
            self.copy(
                &texture,
                None,
                sdl2::rect::Rect::from_center(point_to_sdl2_point(center), width, height),
            )
            .unwrap();
        }
    }
}

pub trait DrawConfinedText {
    type Context;

    fn draw_confined_text(
        &mut self,
        texture_creator: &sdl2::render::TextureCreator<Self::Context>,
        font_provider: &FontProvider,
        text: &str,
        bounding_box: Rect<Float>,
        color: Color,
    );
}

impl<T> DrawConfinedText for sdl2::render::Canvas<T>
where
    T: sdl2::render::RenderTarget,
{
    type Context = T::Context;
    
    fn draw_confined_text(
        &mut self,
        texture_creator: &sdl2::render::TextureCreator<T::Context>,
        font_provider: &FontProvider,
        text: &str,
        bounding_box: Rect<Float>,
        color: Color,
    ) {
        if text.len() > 0 {
            let lines_count = text.lines().count();
            let longest_line_len = text.lines().max_by_key(|line| line.len()).unwrap().len();

            let ssx = (bounding_box.w() / longest_line_len as Float) as u16;
            let ssy = (bounding_box.h() / lines_count as Float) as u16;
            
            let point_size = ssx.min(ssy) * 4 / 3;
            let font = font_provider.font(point_size);

            let surface = font
                .render(text)
                .blended(color_to_sdl2_rgba_color(color))
                .map_err(|e| e.to_string())
                .unwrap();

            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

            let sdl2::render::TextureQuery {
                mut width,
                mut height,
                ..
            } = texture.query();

            width = width.min(*bounding_box.w() as u32);
            height = height.min(*bounding_box.h() as u32);

            self.copy(
                &texture,
                None,
                sdl2::rect::Rect::from_center(
                    point_to_sdl2_point(bounding_box.center()),
                    width,
                    height,
                ),
            )
            .unwrap();
        }
    }
}
