use crate::editor::Editor;
use crate::logger::{LogPiece, MemLogger};
use crate::render::scene_graph::{
    ColumnLayout, ExtColumnLayout, ExtColumnLayoutOptions, ExtRowLayout, ExtRowLayoutOptions,
    GraphicsNode, Text,
};
use crate::render::{Alignment, OLD_DEFAULT_MARGIN, Pix, RenderError, Renderer};
use burbomath::math::{Point, Rect, Size};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::utils::Float;
use std::convert::Into;
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Instant;

struct DrawLittleMan {
    points: &'static [Point<Float>],
    aabb: Rect<Float>,
    color: Color,
}

impl DrawLittleMan {
    fn new(color: Color) -> Self {
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
            aabb: Rect::aabb_from_points(POINTS.deref().iter().cloned()).unwrap(),
            color,
        }
    }
}

impl<T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawLittleMan {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let qx = bounding_box.w() / self.aabb.w();
        let qy = bounding_box.h() / self.aabb.h();

        let q = qx.min(qy);

        let points: Vec<_> = self
            .points
            .iter()
            .map(|x| bounding_box.left_top() + (*x - self.aabb.left_top()) * q)
            .collect();

        renderer.draw_polygon(&points, self.color.clone());

        for p in points {
            renderer.fill_circle(
                p,
                bounding_box.w().min(*bounding_box.h()) / 50.,
                self.color.clone(),
            );
        }
    }

    fn implicit_size(&self) -> Option<Size<Float>> {
        Some(self.aabb.size())
    }
}

struct DrawLog<'a> {
    log: &'a [LogPiece],
    lines_count_limit: usize,
}

impl<'a> DrawLog<'a> {
    fn new(log: &'a [LogPiece], lines_count_limit: usize) -> Self {
        Self {
            log,
            lines_count_limit,
        }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawLog<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        let tail = if self.log.len() < self.lines_count_limit {
            self.log
        } else {
            &self.log[self.log.len() - self.lines_count_limit..]
        };

        Text {
            text: tail
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n"),
            color: Color::black(),
            alignment: Alignment::left(),
            font_height: None,
        }
        .draw(
            renderer,
            bounding_box
                .homogeneous_mul(OLD_DEFAULT_MARGIN.assume_relative().unwrap().value())
                .0,
        );
    }
}

struct DrawHeader<'a> {
    person: &'a Person,
    color: Color,
}

impl<'a> DrawHeader<'a> {
    fn new(person: &'a Person, color: Color) -> Self {
        Self { person, color }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawHeader<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        ColumnLayout::new(vec![
            Box::new(Text {
                color: self.color.clone(),
                alignment: Alignment::left(),
                text: if let Some(boss) = self.person.boss() {
                    format!("{} (B: {})", self.person.name(), boss)
                } else {
                    format!("{}", self.person.name())
                },
                font_height: None,
            }),
            Box::new(Text {
                color: self.color.clone(),
                alignment: Alignment::left(),
                text: format!("{}", self.person.id()),
                font_height: None,
            }),
            Box::new(
                self.person
                    .bank()
                    .map(|bank| format!("{} ({})", self.person.wallet(), bank))
                    .unwrap_or_else(|| format!("{}", self.person.wallet())),
            ),
        ])
        .draw(renderer, bounding_box);
    }
}

struct DrawFooter<'a> {
    person: &'a Person,
}

impl<'a> DrawFooter<'a> {
    fn new(person: &'a Person) -> Self {
        Self { person }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawFooter<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        ColumnLayout::new(vec![
            Box::new(if let Some(objective) = self.person.objective_type_id() {
                format!("{}", objective)
            } else {
                "Idle".into()
            }),
            Box::new(self.person.objective_status().unwrap_or("".into())),
        ])
        .draw(renderer, bounding_box);
    }
}

pub struct PersonRenderModel {
    start_instant: Instant,
}

impl PersonRenderModel {
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        person: &Person,
        editor: &Editor,
        logger: &MemLogger,
        bounding_box: Rect<Float>,
    ) -> Result<(), RenderError> {
        let person_color = match person.state() {
            dudes_in_space_api::person::PersonState::Active => Color::from_uuid(person.id())
                .with_saturation(self.start_instant.elapsed().as_secs_f64().rem_euclid(1.)),
            dudes_in_space_api::person::PersonState::Passive => Color::from_uuid(person.id()),
        };
        if bounding_box.w() > bounding_box.h() {
            let row = ExtRowLayout::new(vec![
                (
                    ExtRowLayoutOptions::preserve_aspect_ratio(),
                    Box::new(DrawLittleMan::new(person_color.clone())),
                ),
                (
                    Default::default(),
                    Box::new(DrawLog::new(
                        logger.get(&person.id()),
                        editor.log_lines_count_limit(),
                    )),
                ),
            ]);

            let column = ExtColumnLayout::new(
                Pix(0.).into(),
                vec![
                    (
                        ExtColumnLayoutOptions::fill_height(),
                        Box::new(DrawHeader::new(person, person_color.clone())),
                    ),
                    (
                        ExtColumnLayoutOptions::fill_height(), /*ExtColumnLayoutOptions::relative_height(0.5)*/
                        Box::new(row),
                    ),
                    (
                        ExtColumnLayoutOptions::fill_height(),
                        Box::new(DrawFooter::new(person)),
                    ),
                ],
            );

            column.draw(renderer, bounding_box);
        } else {
            let column = ExtColumnLayout::new(
                Pix(0.).into(),
                vec![
                    (
                        Default::default(),
                        Box::new(DrawHeader::new(person, person_color.clone())),
                    ),
                    (
                        ExtColumnLayoutOptions::fill_height(),
                        // ExtColumnLayoutOptions::relative_height(0.2),
                        Box::new(DrawLittleMan::new(person_color)),
                    ),
                    (
                        ExtColumnLayoutOptions::fill_height(),
                        // ExtColumnLayoutOptions::relative_height(0.6),
                        Box::new(DrawLog::new(
                            logger.get(&person.id()),
                            editor.log_lines_count_limit(),
                        )),
                    ),
                    (Default::default(), Box::new(DrawFooter::new(person))),
                ],
            );

            column.draw(renderer, bounding_box);
        }

        Ok(())
    }
}
