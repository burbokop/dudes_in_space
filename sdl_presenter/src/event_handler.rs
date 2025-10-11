use crate::AppPaths;
use crate::camera::Camera;
use crate::editor::{Editor, EditorPreset, EditorState};
use crate::logger::MemLogger;
use crate::person_table::PersonTable;
use crate::utils::{save, save_logger};
use crate::vessel_table::VesselTable;
use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::trade::ItemTradeTable;
use dudes_in_space_api::utils::math::Point;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_core::components::Components;
use sdl2::mouse::MouseButton;
use std::ops::ControlFlow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Environment,
    TradeTable,
}

pub(crate) struct EventHandler {
    screen: Screen,
    control: bool,
    shift: bool,
    mouse_position: Point<i32>,
    trade_table_offset: Float,
}

impl EventHandler {
    pub(crate) fn mouse_position(&self) -> Point<i32> {
        self.mouse_position
    }

    pub(crate) fn new() -> Self {
        Self {
            screen: Screen::Environment,
            control: false,
            shift: false,
            mouse_position: (0, 0).into(),
            trade_table_offset: 0.,
        }
    }

    pub(crate) fn trade_table_offset(&self) -> Float {
        self.trade_table_offset
    }

    pub(crate) fn screen(&self) -> Screen {
        self.screen
    }

    fn hover(&self) {
        // TODO
    }

    pub(crate) fn handle_events(
        &mut self,
        environment: &mut Environment,
        trade_table: &mut ItemTradeTable,
        logger: &mut MemLogger,
        event_pump: &mut sdl2::EventPump,
        camera: &mut Camera,
        person_table: &mut PersonTable,
        vessel_table: &mut VesselTable,
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
                        components.item_vault.clone(),
                        &components.subordination_table,
                        &components.bank_registry,
                        &components.wallet_registry,
                        &components.currency_generator,
                        logger,
                    );

                    save(&environment, &paths.save_path);
                    save_logger(&logger, &paths.logger_save_path);
                    *person_table = PersonTable::new(&environment);
                    *vessel_table = VesselTable::new(&environment);
                    *trade_table = ItemTradeTable::build(
                        &components.bank_registry,
                        &components.wallet_registry,
                        environment.vessels(),
                    );
                }

                KeyDown {
                    keycode: Some(Keycode::F1),
                    repeat: false,
                    ..
                } => match self.screen {
                    Screen::Environment => self.screen = Screen::TradeTable,
                    Screen::TradeTable => self.screen = Screen::Environment,
                },

                KeyUp {
                    keycode: Some(Keycode::Comma),
                    ..
                } => match self.screen {
                    Screen::Environment => editor.decrement_log_lines_count_limit(),
                    Screen::TradeTable => {}
                },
                KeyUp {
                    keycode: Some(Keycode::Period),
                    ..
                } => match self.screen {
                    Screen::Environment => editor.increment_log_lines_count_limit(),
                    Screen::TradeTable => {}
                },
                KeyUp {
                    keycode: Some(Keycode::Delete),
                    ..
                } => match self.screen {
                    Screen::Environment => editor.delete_selected(),
                    Screen::TradeTable => {}
                },
                KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => match self.screen {
                    Screen::Environment => editor.confirm_operation(environment),
                    Screen::TradeTable => {}
                },
                KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => match self.screen {
                    Screen::Environment => editor.begin_placing(EditorPreset::PeterCrafter),
                    Screen::TradeTable => {}
                },
                MouseMotion { x, y, .. } => match self.screen {
                    Screen::Environment => {
                        self.mouse_position = (x, y).into();
                        let mouse_position =
                            &(!&camera.transformation()).unwrap() * &self.mouse_position.as_f64();
                        editor.update_nearest_vessel(environment, mouse_position)
                    }
                    Screen::TradeTable => {}
                },
                MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {}
                MouseButtonUp {
                    x, y, mouse_btn, ..
                } => match self.screen {
                    Screen::Environment => {
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
                                    components.item_vault.clone(),
                                    mouse_position,
                                ),
                            },
                            MouseButton::Middle => editor.reset(),
                            MouseButton::Right => editor.reset(),
                            MouseButton::X1 => editor.reset(),
                            MouseButton::X2 => editor.reset(),
                        }
                    }
                    Screen::TradeTable => {}
                },
                MouseWheel {
                    mouse_x,
                    mouse_y,
                    y,
                    ..
                } => {
                    match self.screen {
                        Screen::Environment => {
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
                        Screen::TradeTable => {
                            let velocity: Float = 40.; // px per step
                            self.trade_table_offset += y as Float * velocity;

                            if self.trade_table_offset > 0. {
                                self.trade_table_offset = 0.;
                            }
                        }
                    }
                }

                _ => {}
            }
        }
        ControlFlow::Continue(())
    }
}
