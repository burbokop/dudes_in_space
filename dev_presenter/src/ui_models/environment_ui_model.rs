use burbomath::{Matrix, Rect, Size, camera::Camera};
use dudes_in_space_api::{
    environment::Environment,
    utils::{color::Color, utils::Float},
};
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::{
    editor::{Editor, EditorState},
    event_handler::EventHandler,
    logger::MemLogger,
    person_table::PersonTable,
    render::{
        Alignment, CanvasSurfaceProvider, EnvironmentRenderModel, FontProvider,
        HorisontalAlignment, Renderer, VerticalAlignment,
    },
};

pub struct EnvironmentUiModel<'a> {
    font_provider: &'a FontProvider,
    buffer: &'a mut SharedPixelBuffer<Rgba8Pixel>,
    camera: &'a Camera<Float>,
    environment_render_model: &'a EnvironmentRenderModel,
    environment: &'a Environment,
    editor: &'a Editor,
    logger: &'a MemLogger,
    person_table: &'a PersonTable,
    event_handler: &'a EventHandler,
}

struct DefaultCanvasSurfaceProvider;

impl<'s> CanvasSurfaceProvider<sdl2::surface::Surface<'s>> for DefaultCanvasSurfaceProvider {
    fn surface_mut<'a>(
        &self,
        canvas: &'a mut sdl2::render::Canvas<sdl2::surface::Surface<'s>>,
    ) -> Option<&'a mut sdl2::surface::SurfaceRef> {
        Some(canvas.surface_mut())
    }
}

impl<'a> EnvironmentUiModel<'a> {
    pub fn new(
        font_provider: &'a FontProvider,
        buffer: &'a mut SharedPixelBuffer<Rgba8Pixel>,
        camera: &'a Camera<Float>,
        environment_render_model: &'a EnvironmentRenderModel,
        environment: &'a Environment,
        editor: &'a Editor,
        logger: &'a MemLogger,
        person_table: &'a PersonTable,
        event_handler: &'a EventHandler,
    ) -> Self {
        Self {
            font_provider,
            buffer,
            camera,
            environment_render_model,
            environment,
            editor,
            logger,
            person_table,
            event_handler,
        }
    }

    pub fn apply(&mut self, req: crate::EnvironmentUiRequest) -> crate::EnvironmentUiModelSink {
        // TODO
        let quality_deterioration = 1;

        let requested_view_port_width = req.requested_view_port_width as u32;
        let requested_view_port_height = req.requested_view_port_height as u32;

        if self.buffer.width() != requested_view_port_width
            || self.buffer.height() != requested_view_port_height
        {
            *self.buffer =
                SharedPixelBuffer::new(requested_view_port_width, requested_view_port_height);
            // self.model
            //     .init((self.buffer.width(), self.buffer.height()).into());
        }

        let buffer_size: Size<u32> = (self.buffer.width(), self.buffer.height()).into();
        let view_port_rect: Rect<_> = (
            0.,
            0.,
            (*buffer_size.w() * quality_deterioration) as Float,
            (*buffer_size.h() * quality_deterioration) as Float,
        )
            .into();

        assert_eq!(
            self.buffer.as_bytes().len(),
            self.buffer.width() as usize * self.buffer.height() as usize * 4
        );
        let buffer_size: Size<u32> = (self.buffer.width(), self.buffer.height()).into();

        {
            let surface = sdl2::surface::Surface::from_data(
                self.buffer.make_mut_bytes(),
                *buffer_size.w(),
                *buffer_size.h(),
                *buffer_size.w() * 4,
                sdl2::pixels::PixelFormatEnum::RGBA32,
            )
            .unwrap();

            let canvas = surface.into_canvas().unwrap();

            let texture_creator = canvas.texture_creator();

            let mut renderer = Renderer::new(
                canvas,
                texture_creator,
                &self.font_provider,
                Box::new(DefaultCanvasSurfaceProvider),
            );

            renderer.begin();
            renderer.set_transformation(self.camera.transformation());
            self.environment_render_model
                .render(
                    &mut renderer,
                    &self.environment,
                    &self.editor,
                    &self.logger,
                    &self.person_table,
                )
                .unwrap();

            renderer.set_transformation(Matrix::identity());

            renderer
                .draw_text(
                    &format!("iteration: {}", self.environment.iteration()),
                    (16., 16.).into(),
                    16.,
                    Alignment {
                        horisontal: HorisontalAlignment::Left,
                        vertical: VerticalAlignment::Top,
                    },
                    Color::black(),
                )
                .unwrap();

            match self.editor.state() {
                EditorState::Selection { selected_vessel_id } => {}
                EditorState::Placing { preset_to_place } => {
                    renderer
                        .draw_text(
                            &format!("{:?}", preset_to_place),
                            self.event_handler.mouse_position().as_f64(),
                            16.,
                            Alignment::center(),
                            Color::black(),
                        )
                        .unwrap();
                }
            }

            if self.editor.about_to_delete_selected() {
                let vessel = self
                    .environment
                    .vessel_by_id(self.editor.selected_vessel().unwrap().clone())
                    .unwrap();

                renderer
                    .draw_text(
                        &format!(
                            "Press 'Enter' to confirm deletion of vessel: {} ({})",
                            vessel.name(),
                            vessel.id()
                        ),
                        (16., *renderer.size().h() as Float - 16.).into(),
                        16.,
                        Alignment {
                            horisontal: HorisontalAlignment::Left,
                            vertical: VerticalAlignment::Bottom,
                        },
                        Color::black(),
                    )
                    .unwrap();
            }

            renderer.end();
        }
        crate::EnvironmentUiModelSink {
            canvas: slint::Image::from_rgba8(self.buffer.clone()),
        }
    }
}
