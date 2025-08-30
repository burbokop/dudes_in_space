use crate::logger::{LogPiece, MemLogger};
use crate::render::scene_graph::{
    ColumnLayout, ExtColumnLayout, ExtColumnLayoutOptions, ExtRowLayout, ExtRowLayoutOptions,
    GraphicsNode, Text,
};
use crate::render::{DEFAULT_MARGIN, HorisontalAlignment, RenderError, Renderer};
use dudes_in_space_api::person::Person;
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::{Point, Rect, Size};
use dudes_in_space_api::utils::utils::Float;
use std::convert::Into;
use std::ops::Deref;
use std::sync::LazyLock;

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
}

impl<'a> DrawLog<'a> {
    fn new(log: &'a [LogPiece]) -> Self {
        Self { log }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawLog<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        static MAX_LOG_SIZE: usize = 5;
        let tail = if self.log.len() < MAX_LOG_SIZE {
            self.log
        } else {
            &self.log[self.log.len() - MAX_LOG_SIZE..]
        };

        Text {
            text: tail
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n"),
            color: Color::black(),
            alignment: HorisontalAlignment::Left,
        }
        .draw(renderer, bounding_box.homogeneous_mul(DEFAULT_MARGIN).0);
    }
}

struct DrawHeader<'a> {
    person: &'a Person,
}

impl<'a> DrawHeader<'a> {
    fn new(person: &'a Person) -> Self {
        Self { person }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawHeader<'a> {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        ColumnLayout::new(vec![
            Box::new(Text {
                color: Color::from_uuid(self.person.id()),
                alignment: HorisontalAlignment::Left,
                text: if let Some(boss) = self.person.boss() {
                    format!("{} (B: {})", self.person.name(), boss)
                } else {
                    format!("{}", self.person.name())
                },
            }),
            Box::new(Text {
                color: Color::from_uuid(self.person.id()),
                alignment: HorisontalAlignment::Left,
                text: format!("{}", self.person.id()),
            }),
            Box::new(format!("{:?}", self.person.wallet())),
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
        ColumnLayout::new(vec![Box::new(
            if let Some(objective) = self.person.objective_type_id() {
                format!("{}", objective)
            } else {
                "Idle".into()
            },
        )])
        .draw(renderer, bounding_box);
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
        logger: &MemLogger,
        bounding_box: Rect<Float>,
    ) -> Result<(), RenderError> {
        let person_color = Color::from_uuid(person.id());
        if bounding_box.w() > bounding_box.h() {
            let row = ExtRowLayout::new(vec![
                (
                    Box::new(DrawLittleMan::new(person_color)),
                    ExtRowLayoutOptions::preserve_aspect_ratio(),
                ),
                (
                    Box::new(DrawLog::new(logger.get(&person.id()))),
                    Default::default(),
                ),
            ]);

            let column = ExtColumnLayout::new(vec![
                (Box::new(DrawHeader::new(person)), Default::default()),
                (Box::new(row), ExtColumnLayoutOptions::relative_height(0.5)),
                (Box::new(DrawFooter::new(person)), Default::default()),
            ]);

            column.draw(renderer, bounding_box);
        } else {
            let column = ExtColumnLayout::new(vec![
                (Box::new(DrawHeader::new(person)), Default::default()),
                (
                    Box::new(DrawLittleMan::new(person_color)),
                    ExtColumnLayoutOptions::relative_height(0.2),
                ),
                (
                    Box::new(DrawLog::new(logger.get(&person.id()))),
                    ExtColumnLayoutOptions::relative_height(0.6),
                ),
                (Box::new(DrawFooter::new(person)), Default::default()),
            ]);

            column.draw(renderer, bounding_box);
        }

        Ok(())
    }
}
