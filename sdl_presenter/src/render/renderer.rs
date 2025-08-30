use crate::render::{
    FontProvider, color_to_sdl2_rgba_color, point_to_sdl2_point, rect_to_sdl2_rect,
};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Matrix, Point, Rect};
use dudes_in_space_api::utils::utils::Float;
use sdl2::gfx::primitives::DrawRenderer;
use std::error::Error;
use std::ops::Not;

#[derive(Clone, Copy)]
pub enum HorisontalAlignment {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy)]
pub enum VerticalAlignment {
    Top,
    Bottom,
    Center,
}

#[derive(Clone, Copy)]
pub struct Alignment {
    pub horisontal: HorisontalAlignment,
    pub vertical: VerticalAlignment,
}

impl Alignment {
    pub fn left() -> Self {
        Self {
            horisontal: HorisontalAlignment::Left,
            vertical: VerticalAlignment::Center,
        }
    }

    pub fn right() -> Self {
        Self {
            horisontal: HorisontalAlignment::Right,
            vertical: VerticalAlignment::Center,
        }
    }

    pub fn top() -> Self {
        Self {
            horisontal: HorisontalAlignment::Center,
            vertical: VerticalAlignment::Top,
        }
    }

    pub fn bottom() -> Self {
        Self {
            horisontal: HorisontalAlignment::Center,
            vertical: VerticalAlignment::Bottom,
        }
    }

    pub fn center() -> Self {
        Self {
            horisontal: HorisontalAlignment::Center,
            vertical: VerticalAlignment::Center,
        }
    }
}

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

    pub fn begin(&mut self) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();
    }

    pub fn set_transformation(&mut self, tr: Matrix<Float>) {
        self.tr = tr;
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

    pub fn is_contained_in_view_port(&self, rect: &Point<Float>) -> bool {
        self.view_port_in_world_space.contains_point(rect)
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

    pub fn draw_circle(&mut self, center: Point<Float>, radius: Float, color: Color) {
        let center = point_to_sdl2_point(&self.tr * &center);
        let radius = radius * self.tr.average_scale();
        let _ = self.canvas.circle(
            center.x as i16,
            center.y as i16,
            radius as i16,
            color_to_sdl2_rgba_color(color.clone()),
        );
    }

    pub fn fill_circle(&mut self, center: Point<Float>, radius: Float, color: Color) {
        let center = point_to_sdl2_point(&self.tr * &center);
        let radius = radius * self.tr.average_scale();
        self.canvas
            .filled_circle(
                center.x as i16,
                center.y as i16,
                radius as i16,
                color_to_sdl2_rgba_color(color.clone()),
            )
            .unwrap();
    }

    pub fn draw_polygon(&mut self, polygon: &[Point<Float>], color: Color) {
        if polygon.is_empty() {
            return;
        }

        let polygon: Vec<_> = polygon
            .iter()
            .map(|p| point_to_sdl2_point(&self.tr * p))
            .collect();

        self.canvas.set_draw_color(color_to_sdl2_rgba_color(color));
        self.canvas.draw_lines(polygon.as_slice()).unwrap();
        self.canvas
            .draw_line(polygon[0], polygon[polygon.len() - 1])
            .unwrap();
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        position: Point<Float>,
        point_size: Float,
        alignment: Alignment,
        color: Color,
    ) -> Result<(), Box<dyn Error>> {
        let position = &self.tr * &position;
        let point_size = (self.tr.average_scale() * point_size) as u16;

        if text.len() > 0 {
            let font = self.font_provider.font(point_size);
            let color = color_to_sdl2_rgba_color(color);
            let lines_count = text.lines().count();

            for (i, line) in text.lines().enumerate() {
                let surface = font.render(line).blended(color)?;
                let texture = self.texture_creator.create_texture_from_surface(&surface)?;
                let sdl2::render::TextureQuery { width, height, .. } = texture.query();

                let width = width as Float;
                let height = height as Float;

                let offset_y =
                    (i as isize - lines_count as isize / 2) as Float * font.height() as Float;

                let centered_rect = Rect::from_center(
                    (*position.x(), *position.y() + offset_y).into(),
                    (width, height).into(),
                );

                let rect = match (alignment.horisontal, alignment.vertical) {
                    (HorisontalAlignment::Left, VerticalAlignment::Top) => (
                        *position.x(),
                        *position.y() + offset_y,
                        *centered_rect.w(),
                        *centered_rect.h(),
                    )
                        .into(),
                    (HorisontalAlignment::Center, VerticalAlignment::Top) => todo!(),
                    (HorisontalAlignment::Right, VerticalAlignment::Top) => todo!(),
                    (HorisontalAlignment::Left, VerticalAlignment::Center) => (
                        *position.x(),
                        *centered_rect.y() + offset_y,
                        *centered_rect.w(),
                        *centered_rect.h(),
                    )
                        .into(),
                    (HorisontalAlignment::Center, VerticalAlignment::Center) => centered_rect,
                    (HorisontalAlignment::Right, VerticalAlignment::Center) => todo!(),
                    (HorisontalAlignment::Left, VerticalAlignment::Bottom) => todo!(),
                    (HorisontalAlignment::Center, VerticalAlignment::Bottom) => todo!(),
                    (HorisontalAlignment::Right, VerticalAlignment::Bottom) => todo!(),
                };

                self.canvas.copy(&texture, None, rect_to_sdl2_rect(rect))?;
            }
        }

        Ok(())
    }

    pub fn draw_confined_text(
        &mut self,
        text: &str,
        bounding_box: Rect<Float>,
        alignment: HorisontalAlignment,
        color: Color,
    ) {
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

            self.canvas.set_draw_color(color);
            for (i, line) in text.lines().enumerate() {
                let surface = font.render(line).blended(color);

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

                let sdl2::render::TextureQuery { width, height, .. } = texture.query();

                let mut width = width as Float;
                let mut height = height as Float;

                width = width.min(*bounding_box.w());
                height = height.min(*bounding_box.h());

                let offset_y =
                    (i as isize - lines_count as isize / 2) as Float * font.height() as Float;

                let centered_rect = Rect::from_center(
                    (
                        *bounding_box.center().x(),
                        *bounding_box.center().y() + offset_y,
                    )
                        .into(),
                    (width, height).into(),
                );

                let rect = match alignment {
                    HorisontalAlignment::Left => (
                        *bounding_box.x(),
                        *centered_rect.y(),
                        *centered_rect.w(),
                        *centered_rect.h(),
                    )
                        .into(),
                    HorisontalAlignment::Right => todo!(),
                    HorisontalAlignment::Center => centered_rect,
                };

                self.canvas
                    .copy(&texture, None, rect_to_sdl2_rect(rect))
                    .unwrap();
            }
        }
    }
}
