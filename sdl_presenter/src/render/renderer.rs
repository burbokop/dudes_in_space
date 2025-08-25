use crate::camera::Camera;
use crate::render::{
    FontProvider, color_to_sdl2_rgba_color, point_to_sdl2_point, rect_to_sdl2_rect,
};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Matrix, Rect};
use dudes_in_space_api::utils::utils::Float;
use std::ops::Not;

pub struct Renderer<T: sdl2::render::RenderTarget> {
    canvas: sdl2::render::Canvas<T>,
    texture_creator: sdl2::render::TextureCreator<T::Context>,
    font_provider: FontProvider,
    tr: Matrix<Float>,
    view_port_in_world_space: Rect<Float>,
}

impl<T: sdl2::render::RenderTarget> Renderer<T> {
    pub fn new(
        canvas: sdl2::render::Canvas<T>,
        texture_creator: sdl2::render::TextureCreator<T::Context>,
        font_provider: FontProvider,
    ) -> Self {
        Self {
            canvas,
            texture_creator,
            font_provider,
            tr: Matrix::identity(),
            view_port_in_world_space: (0., 0., 0., 0.).into(),
        }
    }

    pub fn begin(&mut self, camera: &Camera) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();

        self.tr = camera.transformation();

        let (canvas_width, canvas_height) = self.canvas.output_size().unwrap();
        let view_port: Rect<Float> = (0., 0., canvas_width as Float, canvas_height as Float).into();

        self.view_port_in_world_space = &self.tr.not().unwrap() * &view_port;
    }

    pub fn end(&mut self) {
        self.canvas.present();
    }

    pub fn intersects_with_view_port(&self, rect: &Rect<Float>) -> bool {
        self.view_port_in_world_space.instersects(rect)
    }

    pub fn draw_rect(&mut self, rect: Rect<Float>, color: Color) {
        if !self.intersects_with_view_port(&rect) {
            return;
        }

        self.canvas.set_draw_color(color_to_sdl2_rgba_color(color));
        self.canvas
            .draw_rect(rect_to_sdl2_rect(&self.tr * &rect))
            .unwrap();
    }

    pub fn draw_confined_text(&mut self, text: &str, bounding_box: Rect<Float>, color: Color) {
        if !self.intersects_with_view_port(&bounding_box) {
            return;
        }

        let bounding_box = &self.tr * &bounding_box;

        if text.len() > 0 {
            let lines_count = text.lines().count();
            let longest_line_len = text.lines().max_by_key(|line| line.len()).unwrap().len();

            let ssx = (bounding_box.w() / longest_line_len as Float) as u16;
            let ssy = (bounding_box.h() / lines_count as Float) as u16;

            let point_size = ssx.min(ssy).checked_mul(4);
            if point_size.is_none() {
                self.canvas
                    .fill_rect(rect_to_sdl2_rect(bounding_box))
                    .unwrap();
                return;
            }

            let point_size = point_size.unwrap() / 3;

            let font = self.font_provider.font(point_size);

            let color = color_to_sdl2_rgba_color(color);

            let surface = font.render(text).blended(color);

            self.canvas.set_draw_color(color);
            if surface.is_err() {
                self.canvas
                    .fill_rect(rect_to_sdl2_rect(bounding_box))
                    .unwrap();
                return;
            }
            let surface = surface.unwrap();

            let texture = self.texture_creator.create_texture_from_surface(&surface);
            if texture.is_err() {
                self.canvas
                    .fill_rect(rect_to_sdl2_rect(bounding_box))
                    .unwrap();
                return;
            }
            let texture = texture.unwrap();

            let sdl2::render::TextureQuery {
                mut width,
                mut height,
                ..
            } = texture.query();

            width = width.min(*bounding_box.w() as u32);
            height = height.min(*bounding_box.h() as u32);

            self.canvas
                .copy(
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
