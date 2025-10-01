use crate::AppPaths;
use crate::camera::Camera;
use crate::editor::{Editor, EditorPreset, EditorState};
use crate::logger::MemLogger;
use crate::person_table::PersonTable;
use crate::utils::{save, save_logger};
use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::utils::math::Point;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_core::components::Components;
use sdl2::mouse::MouseButton;
use std::ops::ControlFlow;

pub(crate) struct EventHandler {
    control: bool,
    shift: bool,
    mouse_position: Point<i32>,
}

impl EventHandler {
    pub(crate) fn mouse_position(&self) -> Point<i32> {
        self.mouse_position
    }

    pub(crate) fn new() -> Self {
        Self {
            control: false,
            shift: false,
            mouse_position: (0, 0).into(),
        }
    }

    fn hover(&self) {
        // TODO
    }

    pub(crate) fn handle_events(
        &mut self,
        environment: &mut Environment,
        logger: &mut MemLogger,
        event_pump: &mut sdl2::EventPump,
        camera: &mut Camera,
        person_table: &mut PersonTable,
        editor: &mut Editor,
        components: &Components,
        paths: &AppPaths,
    ) -> ControlFlow<()> {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event::{
                KeyDown, KeyUp, MouseButtonDown, MouseButtonUp, MouseMotion, MouseWheel, Quit,
            };
            use sdl2::keyboard::Keycode;
            match event {
                Quit { .. }
                | KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return ControlFlow::Break(()),
                KeyDown {
                    keycode: Some(Keycode::LShift | Keycode::RShift),
                    ..
                } => self.shift = true,
                KeyUp {
                    keycode: Some(Keycode::LShift | Keycode::RShift),
                    ..
                } => self.shift = false,
                KeyDown {
                    keycode: Some(Keycode::LCtrl | Keycode::RCtrl),
                    ..
                } => self.control = true,
                KeyUp {
                    keycode: Some(Keycode::LCtrl | Keycode::RCtrl),
                    ..
                } => self.control = false,

                KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    environment.proceed(
                        &components.process_token_context,
                        &components.req_context,
                        &components.objectives_decider_vault,
                        &components.item_vault,
                        &components.subordination_table,
                        &components.bank_registry,
                        &components.wallet_registry,
                        &components.currency_generator,
                        logger,
                    );

                    save(&environment, &paths.save_path);
                    save_logger(&logger, &paths.logger_save_path);
                    *person_table = PersonTable::new(&environment)
                }
                KeyUp {
                    keycode: Some(Keycode::Comma),
                    ..
                } => editor.decrement_log_lines_count_limit(),
                KeyUp {
                    keycode: Some(Keycode::Period),
                    ..
                } => editor.increment_log_lines_count_limit(),
                KeyUp {
                    keycode: Some(Keycode::Delete),
                    ..
                } => editor.delete_selected(),
                KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => editor.confirm_operation(environment),
                KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => editor.begin_placing(EditorPreset::PeterCrafter),
                MouseMotion { x, y, .. } => {
                    self.mouse_position = (x, y).into();
                    let mouse_position =
                        &(!&camera.transformation()).unwrap() * &self.mouse_position.as_f64();
                    editor.update_nearest_vessel(environment, mouse_position)
                }
                MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {}
                MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    let mouse_position =
                        &(!&camera.transformation()).unwrap() * &self.mouse_position.as_f64();
                    match mouse_btn {
                        MouseButton::Unknown => unreachable!(),
                        MouseButton::Left => match editor.state() {
                            EditorState::Selection { .. } => {
                                editor.select_hearest_vessel(environment)
                            }
                            EditorState::Placing { .. } => editor.end_placing(
                                environment,
                                &components.item_vault,
                                mouse_position,
                            ),
                        },
                        MouseButton::Middle => editor.reset(),
                        MouseButton::Right => editor.reset(),
                        MouseButton::X1 => editor.reset(),
                        MouseButton::X2 => editor.reset(),
                    }
                }
                MouseWheel {
                    mouse_x,
                    mouse_y,
                    y,
                    ..
                } => {
                    let angle_delta_to_scale_division = |angle_delta: Float| {
                        let base: Float = 1.2;

                        base.powf(angle_delta)
                    };

                    let angle_delta_to_translation_delta = |angle_delta: Float| {
                        let velocity: Float = 10.; // px per step
                        return velocity * angle_delta;
                    };

                    let position = (mouse_x as Float, mouse_y as Float).into();

                    if self.control {
                        // zoom
                        camera.concat_scale_centered(
                            angle_delta_to_scale_division(y as Float),
                            position,
                            position,
                        );
                    } else if self.shift {
                        // scroll horizontally
                        camera.add_translation(
                            (angle_delta_to_translation_delta(y as Float), 0.).into(),
                        );
                    } else {
                        // scroll vertically
                        camera.add_translation(
                            (0., angle_delta_to_translation_delta(y as Float)).into(),
                        );
                    }
                }

                _ => {}
            }
        }
        ControlFlow::Continue(())
    }
}
