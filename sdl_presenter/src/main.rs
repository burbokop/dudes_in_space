#![feature(fn_traits)]
#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::camera::Camera;
use crate::render::{
    Alignment, EnvironmentRenderModel, FontProvider, HorisontalAlignment, Renderer,
    VerticalAlignment,
};
use crate::utils::{load, load_camera, load_logger, save, save_camera, save_logger};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_core::components::core_components;
use std::env::home_dir;
use std::time::Duration;

mod camera;
mod logger;
mod render;
mod utils;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let mut control = false;
    let mut shift = false;

    let save_path = home_dir().unwrap().join(".dudes_in_space/save.json");
    let camera_save_path = home_dir().unwrap().join(".dudes_in_space/camera.json");
    let logger_save_path = home_dir().unwrap().join(".dudes_in_space/logger.json");

    let mut camera: Camera = load_camera(camera_save_path.clone());
    let mut logger = load_logger(logger_save_path.clone());
    let render_model = EnvironmentRenderModel::new();
    let font_provider = FontProvider::new();
    let texture_creator = canvas.texture_creator();

    let mut renderer = Renderer::new(canvas, texture_creator, font_provider);

    let components = core_components();
    let mut environment = load(&components, save_path.clone());
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event::{KeyDown, KeyUp, MouseWheel, Quit};
            use sdl2::keyboard::Keycode;
            match event {
                Quit { .. }
                | KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                KeyDown {
                    keycode: Some(Keycode::LShift | Keycode::RShift),
                    ..
                } => shift = true,
                KeyUp {
                    keycode: Some(Keycode::LShift | Keycode::RShift),
                    ..
                } => shift = false,
                KeyDown {
                    keycode: Some(Keycode::LCtrl | Keycode::RCtrl),
                    ..
                } => control = true,
                KeyUp {
                    keycode: Some(Keycode::LCtrl | Keycode::RCtrl),
                    ..
                } => control = false,

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
                        &mut logger,
                    );
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

                    if control {
                        // zoom
                        camera.concat_scale_centered(
                            angle_delta_to_scale_division(y as Float),
                            position,
                            position,
                        );
                    } else if shift {
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

        renderer.begin(&camera);
        render_model
            .render(&mut renderer, &environment, &logger)
            .unwrap();

        renderer.draw_text(
            &format!("iteration: {}", environment.iteration()),
            (16., 16.).into(),
            16,
            Alignment {
                horisontal: HorisontalAlignment::Left,
                vertical: VerticalAlignment::Top,
            },
            Color::black(),
        );

        renderer.end();

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
    save(environment, save_path);
    save_camera(camera, camera_save_path);
    save_logger(logger, logger_save_path);
}
