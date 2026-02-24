use crate::app::SavePaths;
use crate::editor::{Editor, EditorPreset, EditorState};
use crate::logger::MemLogger;
use crate::person_table::PersonTable;
use crate::utils::{save, save_logger};
use crate::vessel_table::VesselTable;
use burbomath::camera::Camera;
use burbomath::{Point, Vector};
use dudes_in_space_api::environment::Environment;
use dudes_in_space_api::trade::ItemTradeTable;
use dudes_in_space_api::utils::utils::{AsFloat as _, Float};
use dudes_in_space_core::components::Components;
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

pub(crate) enum KeyCode {
    Escape,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    Space,
    F1,
    Comma,
    Period,
    Delete,
    Return,
    Num1,
}

pub(crate) enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

pub(crate) enum Event {
    Quit,
    KeyDown {
        keycode: KeyCode,
    },
    KeyUp {
        keycode: KeyCode,
    },
    MouseMotion {
        position: Point<i32>,
    },
    MouseButtonDown {
        position: Point<i32>,
        button: MouseButton,
    },
    MouseButtonUp {
        position: Point<i32>,
        button: MouseButton,
    },
    MouseWheel {
        position: Point<i32>,
        delta: Vector<i32>,
    },
    MouseCancel,
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

    pub(crate) fn handle_event(
        &mut self,
        environment: &mut Environment,
        trade_table: &mut ItemTradeTable,
        logger: &mut MemLogger,
        camera: &mut Camera<Float>,
        person_table: &mut PersonTable,
        vessel_table: &mut VesselTable,
        editor: &mut Editor,
        components: &Components,
        paths: &SavePaths,
        event: Event,
    ) -> ControlFlow<()> {
        use crate::event_handler::Event::{
            KeyDown, KeyUp, MouseButtonDown, MouseButtonUp, MouseMotion, MouseWheel, Quit,
        };

        match event {
            Quit
            | KeyDown {
                keycode: KeyCode::Escape,
            } => return ControlFlow::Break(()),
            KeyDown {
                keycode: KeyCode::LShift | KeyCode::RShift,
            } => self.shift = true,
            KeyUp {
                keycode: KeyCode::LShift | KeyCode::RShift,
            } => self.shift = false,
            KeyDown {
                keycode: KeyCode::LCtrl | KeyCode::RCtrl,
            } => self.control = true,
            KeyUp {
                keycode: KeyCode::LCtrl | KeyCode::RCtrl,
            } => self.control = false,

            KeyUp {
                keycode: KeyCode::Space,
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
                keycode: KeyCode::F1,
            } => match self.screen {
                Screen::Environment => self.screen = Screen::TradeTable,
                Screen::TradeTable => self.screen = Screen::Environment,
            },

            KeyUp {
                keycode: KeyCode::Comma,
            } => match self.screen {
                Screen::Environment => editor.decrement_log_lines_count_limit(),
                Screen::TradeTable => {}
            },
            KeyUp {
                keycode: KeyCode::Period,
            } => match self.screen {
                Screen::Environment => editor.increment_log_lines_count_limit(),
                Screen::TradeTable => {}
            },
            KeyUp {
                keycode: KeyCode::Delete,
            } => match self.screen {
                Screen::Environment => editor.delete_selected(),
                Screen::TradeTable => {}
            },
            KeyUp {
                keycode: KeyCode::Return,
            } => match self.screen {
                Screen::Environment => editor.confirm_operation(environment),
                Screen::TradeTable => {}
            },
            KeyUp {
                keycode: KeyCode::Num1,
            } => match self.screen {
                Screen::Environment => editor.begin_placing(EditorPreset::PeterCrafter),
                Screen::TradeTable => {}
            },
            MouseMotion { position } => match self.screen {
                Screen::Environment => {
                    self.mouse_position = position;
                    let mouse_position =
                        &(!&camera.transformation()).unwrap() * &self.mouse_position.as_f64();
                    editor.update_nearest_vessel(environment, mouse_position)
                }
                Screen::TradeTable => {}
            },
            MouseButtonDown { position, button } => {}
            MouseButtonUp { position, button } => match self.screen {
                Screen::Environment => {
                    let mouse_position =
                        &(!&camera.transformation()).unwrap() * &self.mouse_position.as_f64();
                    match button {
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
            MouseWheel { position, delta } => {
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

                        if self.control {
                            // zoom
                            camera.concat_scale_centered(
                                angle_delta_to_scale_division(*delta.y() as Float),
                                position.as_float(),
                                position.as_float(),
                            );
                        } else if self.shift {
                            // scroll horizontally
                            camera.add_translation(
                                (angle_delta_to_translation_delta(*delta.y() as Float), 0.).into(),
                            );
                        } else {
                            // scroll vertically
                            camera.add_translation(
                                (0., angle_delta_to_translation_delta(*delta.y() as Float)).into(),
                            );
                        }
                    }
                    Screen::TradeTable => {
                        let velocity: Float = 40.; // px per step
                        self.trade_table_offset += *delta.y() as Float * velocity;

                        if self.trade_table_offset > 0. {
                            self.trade_table_offset = 0.;
                        }
                    }
                }
            }

            _ => {}
        }
        ControlFlow::Continue(())
    }
}
