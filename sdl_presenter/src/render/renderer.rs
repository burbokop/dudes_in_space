use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::Rect;
use dudes_in_space_api::utils::utils::Float;
use crate::camera::Camera;
use crate::render::{color_to_sdl2_rgba_color, point_to_sdl2_point, rect_to_sdl2_rect, FontProvider};

pub struct Renderer {
    canvas: sdl2::render::Canvas<T>,
    texture_creator: sdl2::render::TextureCreator<T::Context>,
    font_provider: FontProvider,
    camera: Camera,
    view_port_in_world_space: Rect<Float>,
}

impl Renderer {
    pub fn new(
        canvas: sdl2::render::Canvas<T>,
        texture_creator: sdl2::render::TextureCreator<T::Context>,
        font_provider: FontProvider,
        camera: Camera,
        view_port_in_world_space: Rect<Float>,
    ) -> Self {
        Self {
            canvas,
            texture_creator,
            font_provider,
            camera,
            view_port_in_world_space,
        }
    }
    
    pub fn draw_confined_text(
        &mut self,
        text: &str,
        bounding_box: Rect<Float>,
        color: Color,
    ) {
        if text.len() > 0 {
            let lines_count = text.lines().count();
            let longest_line_len = text.lines().max_by_key(|line| line.len()).unwrap().len();

            let ssx = (bounding_box.w() / longest_line_len as Float) as u16;
            let ssy = (bounding_box.h() / lines_count as Float) as u16;



            let point_size = ssx.min(ssy) .checked_mul(4) ;
            if point_size.is_none() {
                self.fill_rect(rect_to_sdl2_rect(bounding_box)).unwrap();
                return;
            }

            let point_size = point_size.unwrap() / 3;

            let font = font_provider.font(point_size);

            let color = color_to_sdl2_rgba_color(color);

            let surface = font.render(text).blended(color);

            self.set_draw_color(color);
            if surface.is_err() {
                self.fill_rect(rect_to_sdl2_rect(bounding_box)).unwrap();
                return;
            }
            let surface = surface.unwrap();

            let texture = texture_creator.create_texture_from_surface(&surface);
            if texture.is_err() {
                self.fill_rect(rect_to_sdl2_rect(bounding_box)).unwrap();
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
