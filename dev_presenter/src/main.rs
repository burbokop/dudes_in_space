#![feature(fn_traits)]
#![feature(map_try_insert)]
#![feature(const_trait_impl)]
#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(const_convert)]
#![feature(duration_millis_float)]

use crate::app::App;
use crate::event_handler::{Event, KeyCode, MouseButton};
use crate::utils::save_camera;
use burbomath::LerpIntegrator;
use clap::Parser;
use dudes_in_space_api::utils::utils::Float;
use slint::{
    CloseRequestResponse, ComponentHandle, Model, PlatformError, SharedString, Timer, TimerMode,
    ToSharedString,
};
use std::ops::ControlFlow;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod app;
mod editor;
mod event_handler;
mod logger;
mod person_table;
mod render;
mod ui_models;
mod utils;
mod vessel_table;

slint::slint! {
    export { MoneyUiModelSinkUtils } from "src/ui/sinks.slint";
    export { MainWindow } from "src/ui/main.slint";
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum Renderer {
    Sdl,
    Vulkan,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "sdl")]
    renderer: Renderer,
}

pub fn main() -> Result<(), PlatformError> {
    let args = Args::parse();
    match args.renderer {
        Renderer::Sdl => {}
        Renderer::Vulkan => todo!(),
    }

    let (ctrl_c_tx, ctrl_c_rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        ctrl_c_tx
            .send(())
            .expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    let app = App::new();

    let timer = Rc::new(Timer::default());
    let mut last_tick_instant = Instant::now();

    {
        let weak_app = Rc::downgrade(&app);
        timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(1000 / 30),
            move || {
                let now = Instant::now();
                let dt = now - last_tick_instant;
                last_tick_instant = now;
                let app = weak_app.upgrade().unwrap();
                let mut app = app.borrow_mut();

                match app.proceed() {
                    ControlFlow::Continue(_) => {}
                    ControlFlow::Break(_) => todo!(),
                }
            },
        );
    }

    let weak_timer = Rc::downgrade(&timer);
    let set_desired_tps = move |app: &mut App, tps: Float| {
        let timer = weak_timer.upgrade().unwrap();
        timer.set_interval(std::time::Duration::from_millis((1000. / tps) as u64));
        app.desired_tps = tps;
    };

    let main_window = MainWindow::new().unwrap();

    main_window
        .global::<MoneyUiModelSinkUtils>()
        .on_money_list_to_string(|x| {
            let str = x
                .iter()
                .map(|x| {
                    if x.currency.len() == 1 {
                        return format!("{}{}", x.amount, x.currency);
                    } else {
                        return format!("{} {}", x.amount, x.currency);
                    }
                })
                .collect::<Vec<String>>();

            format!("{:?}", str).to_shared_string()
        });

    // {
    //     let weak_app: std::rc::Weak<_, _> = Rc::downgrade(&app);
    //     main_window.on_tool_clicked(move |tool: DisplayTool| {
    //         let app = weak_app.upgrade().unwrap();
    //         let mut app = app.try_borrow_mut().unwrap();
    //         app.active_tool = tool.into();
    //     })
    // }

    {
        let weak_app = Rc::downgrade(&app);
        main_window.on_pointer_event(move |event_type, button, x: f32, y: f32| {
            let app = weak_app.upgrade().unwrap();
            let mut app = app.try_borrow_mut().unwrap();

            let button = if button == 0 {
                MouseButton::Left
            } else if button == 1 {
                MouseButton::Right
            } else {
                unreachable!()
            };

            let event = if event_type == 0 {
                Event::MouseButtonUp {
                    position: (x as i32, y as i32).into(),
                    button,
                }
            } else if event_type == 1 {
                Event::MouseButtonDown {
                    position: (x as i32, y as i32).into(),
                    button,
                }
            } else if event_type == 2 {
                Event::MouseMotion {
                    position: (x as i32, y as i32).into(),
                }
            } else if event_type == 3 {
                Event::MouseCancel
            } else {
                unreachable!()
            };

            match app.handle_event(event) {
                ControlFlow::Continue(_) => {}
                ControlFlow::Break(_) => todo!(),
            }
        });
    }

    {
        let weak_app = Rc::downgrade(&app);
        main_window.on_scroll_event(move |pos_x, pos_y, delta_x, delta_y, shift, control| {
            let app = weak_app.upgrade().unwrap();
            let mut app = app.try_borrow_mut().unwrap();

            let event = Event::MouseWheel {
                position: (pos_x as i32, pos_y as i32).into(),
                delta: (delta_x as i32 / 120, delta_y as i32 / 120).into(),
            };

            match app.handle_event(event) {
                ControlFlow::Continue(_) => true,
                ControlFlow::Break(_) => todo!(),
            }
        });
    }

    fn keycode_from_text(text: SharedString) -> KeyCode {
        let f1 = [0xEF, 0x9C, 0x84];
        let f2 = [0xEF, 0x9C, 0x85];
        let f3 = [0xEF, 0x9C, 0x86];

        if text == "todo" {
            KeyCode::Escape
        } else if text.as_bytes() == [16] {
            KeyCode::LShift
        } else if text == "todo" {
            KeyCode::RShift
        } else if text.as_bytes() == [17] {
            KeyCode::LCtrl
        } else if text == "todo" {
            KeyCode::RCtrl
        } else if text.as_bytes() == [32] {
            KeyCode::Space
        } else if text.as_bytes() == f1 {
            KeyCode::F1
        } else if text == "todo" {
            KeyCode::Comma
        } else if text == "todo" {
            KeyCode::Period
        } else if text == "todo" {
            KeyCode::Delete
        } else if text == "todo" {
            KeyCode::Return
        } else if text == "todo" {
            KeyCode::Num1
        } else {
            panic!("Unsupported key: {:?}", text.as_str().as_bytes())
        }
    }

    {
        let weak_app = Rc::downgrade(&app);
        main_window.on_key_press_event(move |text| {
            let app = weak_app.upgrade().unwrap();
            let mut app = app.try_borrow_mut().unwrap();

            let event = Event::KeyDown {
                keycode: keycode_from_text(text),
            };

            match app.handle_event(event) {
                ControlFlow::Continue(_) => true,
                ControlFlow::Break(_) => todo!(),
            }
        });
    }

    {
        let weak_app = Rc::downgrade(&app);
        main_window.on_key_release_event(move |text| {
            let app = weak_app.upgrade().unwrap();
            let mut app = app.try_borrow_mut().unwrap();

            let event = Event::KeyUp {
                keycode: keycode_from_text(text),
            };

            match app.handle_event(event) {
                ControlFlow::Continue(_) => true,
                ControlFlow::Break(_) => todo!(),
            }
        });
    }
    main_window.invoke_init_focus();

    let mut prev_render_instant = Instant::now();

    let render_timer = Timer::default();

    {
        let desired_fps = match args.renderer {
            Renderer::Sdl => 30,
            Renderer::Vulkan => 15,
        };

        let render_interval = Duration::from_millis(1000 / desired_fps);

        let weak_app = Rc::downgrade(&app);
        let weak_window = main_window.as_weak();
        let mut fps_integrator: LerpIntegrator<Float, Float> = LerpIntegrator::new(0.2);
        let mut tps_integrator: LerpIntegrator<Float, Float> = LerpIntegrator::new(0.2);
        render_timer.start(TimerMode::Repeated, render_interval, move || {
            if let Some(window) = weak_window.upgrade() {
                let now = Instant::now();
                let dt = now - prev_render_instant;
                prev_render_instant = now;

                let app = weak_app.upgrade().unwrap();
                let mut app = app.borrow_mut();

                // let mut environment_render_model = app.environment_render_model.borrow_mut();

                window.set_sink(app.root_ui_model().apply(window.get_request()));

                // window.set_env_info(EnvInfo {
                //     now: pretty_duration(
                //         app.environment
                //             .now()
                //             .duration_since(app.environment.creation_time()),
                //     )
                //     .into(),
                //     pause: app.pause,
                //     time_speed: app.time_speed as f32,
                //     bugs_count: app.environment.bugs_count() as i32,
                //     food_count: app.environment.food_count() as i32,
                // });
                window.set_fps(*fps_integrator.proceed(1. / dt.as_secs_f64()) as f32);
                window.set_tps(*tps_integrator.proceed(app.tps) as f32);
                window.set_desired_fps(desired_fps as f32);
                window.set_desired_tps(app.desired_tps as f32);
                // window.set_quality_deterioration(app.quality_deterioration as i32);

                // window.set_active_tool(app.active_tool.into());

                // if let Some(bug) = app
                //     .selected_bug_id
                //     .and_then(|id| app.environment.find_bug_by_id(id))
                // {
                //     window.set_selected_bug_info(BugInfo {
                //         genes: bug
                //             .chromosome()
                //             .genes
                //             .iter()
                //             .map(|x| *x as f32)
                //             .collect::<Vec<_>>()[..]
                //             .into(),
                //         age: bug.age(app.environment.now().clone()).unwrap() as f32,
                //         baby_charge_level: bug.baby_charge_level().unwrap() as f32,
                //         baby_charge_capacity: bug.baby_charge_capacity().unwrap() as f32,
                //         color: color_to_slint_rgba_f32_color(bug.color()).into(),
                //         energy_level: bug.energy_level().unwrap() as f32,
                //         energy_capacity: bug.energy_capacity().unwrap() as f32,
                //         id: bug.id() as i32,
                //         rotation: bug.rotation().degrees() as f32,
                //         size: bug.size().unwrap() as f32,
                //         x: *bug.position().x() as f32,
                //         y: *bug.position().y() as f32,
                //         heat_capacity: bug.heat_capacity().unwrap() as f32,
                //         heat_level: bug.heat_level().unwrap() as f32,
                //         vision_range: bug.vision_range().unwrap() as f32,
                //         vision_arc: (bug.vision_half_arc().unwrap().degrees() * 2.) as f32,
                //     });

                //     if app.do_render {
                //         if let Some(brain_log) = bug.last_brain_log() {
                //             let mut brain_render_model = app.brain_render_model.borrow_mut();

                //             window.set_brain_canvas(brain_render_model.render(
                //                 bug.brain(),
                //                 brain_log,
                //                 app.selected_node,
                //                 window.get_requested_brain_canvas_width() as u32,
                //                 window.get_requested_brain_canvas_height() as u32,
                //             ));

                //             window.set_selected_bug_last_brain_log(BugBrainLog {
                //                 input: BugBrainInput {
                //                     color_of_nearest_bug: color_to_slint_rgba_f32_color(
                //                         &brain_log
                //                             .input
                //                             .nearest_bug
                //                             .as_ref()
                //                             .map(|x| x.color.clone())
                //                             .unwrap_or(Color {
                //                                 a: 0.,
                //                                 r: 0.,
                //                                 g: 0.,
                //                                 b: 0.,
                //                             }),
                //                     )
                //                     .into(),
                //                     direction_to_nearest_bug: brain_log
                //                         .input
                //                         .nearest_bug
                //                         .as_ref()
                //                         .map(|x| x.direction)
                //                         .unwrap_or(Angle::from_radians(0.))
                //                         .degrees()
                //                         as f32,
                //                     direction_to_nearest_food: brain_log
                //                         .input
                //                         .nearest_food
                //                         .as_ref()
                //                         .map(|x| x.direction)
                //                         .unwrap_or(Angle::from_radians(0.))
                //                         .degrees()
                //                         as f32,
                //                     rotation: brain_log.input.rotation.degrees() as f32,
                //                     proximity_to_bug: brain_log
                //                         .input
                //                         .nearest_bug
                //                         .as_ref()
                //                         .map(|x| x.dst)
                //                         .unwrap_or(noneg_float(1.))
                //                         .unwrap()
                //                         as f32,
                //                     proximity_to_food: brain_log
                //                         .input
                //                         .nearest_food
                //                         .as_ref()
                //                         .map(|x| x.dst)
                //                         .unwrap_or(noneg_float(1.))
                //                         .unwrap()
                //                         as f32,
                //                 },
                //                 output: BugBrainOutput {
                //                     baby_charging_rate: brain_log.output.baby_charging_rate.unwrap()
                //                         as f32,
                //                     desired_rotation: (bug.rotation()
                //                         + brain_log.output.relative_desired_rotation)
                //                         .degrees()
                //                         as f32,
                //                     rotation_velocity: brain_log
                //                         .output
                //                         .rotation_velocity
                //                         .unwrap()
                //                         .degrees()
                //                         as f32,
                //                     velocity: brain_log.output.velocity as f32,
                //                 },
                //             });
                //         }
                //     }
                // }

                window.window().request_redraw();

                if let Ok(_) = ctrl_c_rx.try_recv() {
                    save_camera(&app.camera, &app.save_paths.camera_save_path);
                    window.window().hide().unwrap();
                }
            }
        });
    }

    {
        let weak_app = Rc::downgrade(&app);
        main_window
            .window()
            .on_close_requested(move || -> CloseRequestResponse {
                let app = weak_app.upgrade().unwrap();
                let app = app.borrow();
                save_camera(&app.camera, &app.save_paths.camera_save_path);
                CloseRequestResponse::HideWindow
            });
    }

    main_window.on_inv_color(|color| {
        slint::Color::from_argb_u8(
            color.alpha(),
            255 - color.red(),
            255 - color.green(),
            255 - color.blue(),
        )
    });

    main_window.run()
}
