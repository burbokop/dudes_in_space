#![feature(fn_traits)]
#![feature(map_try_insert)]
#![deny(warnings)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::camera::Camera;
use crate::editor::{Editor, EditorState};
use crate::event_handler::EventHandler;
use crate::person_table::PersonTable;
use crate::render::{
    Alignment, EnvironmentRenderModel, FontProvider, HorisontalAlignment,
    ModuleTextureContainerBuilder, Renderer, VerticalAlignment,
};
use crate::utils::{load, load_camera, load_logger, save_camera};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::math::Matrix;
use dudes_in_space_api::utils::utils::Float;
use dudes_in_space_core::components::core_components;
use dudes_in_space_core::module_types;
use std::env::home_dir;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::time::Duration;

mod camera;
mod editor;
mod event_handler;
mod logger;
mod person_table;
mod render;
mod utils;

struct AppPaths {
    save_path: PathBuf,
    camera_save_path: PathBuf,
    logger_save_path: PathBuf,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 1600, 800)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let app_paths = AppPaths {
        save_path: home_dir().unwrap().join(".dudes_in_space/save.json"),
        camera_save_path: home_dir().unwrap().join(".dudes_in_space/camera.json"),
        logger_save_path: home_dir().unwrap().join(".dudes_in_space/logger.json"),
    };

    let mut camera: Camera = load_camera(app_paths.camera_save_path.clone());
    let mut logger = load_logger(app_paths.logger_save_path.clone());
    let texture_creator = canvas.texture_creator();
    let module_bg_tex_container = ModuleTextureContainerBuilder::new(&texture_creator)
        .with(
            module_types::DOCKYARD.into(),
            include_bytes!("../assets/dockyard.png"),
        )
        .with_aseprite_spritesheet(
            module_types::FABRICATOR.into(),
            include_bytes!("../assets/fabricator.png"),
            include_bytes!("../assets/fabricator.json"),
        )
        .with_aseprite_spritesheet(
            module_types::ASSEMBLER.into(),
            include_bytes!("../assets/assembler.png"),
            include_bytes!("../assets/assembler.json"),
        )
        .with(
            module_types::PERSONNEL_AREA.into(),
            include_bytes!("../assets/personnel_area.png"),
        )
        .build();

    let render_model = EnvironmentRenderModel::new(module_bg_tex_container.get_ref());
    let font_provider = FontProvider::new();
    let mut renderer = Renderer::new(canvas, &texture_creator, font_provider);
    let components = core_components();
    let mut environment = load(&components, app_paths.save_path.clone());
    let mut person_table = PersonTable::new(&environment);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut event_handler = EventHandler::new();
    let mut editor = Editor::new();

    'running: loop {
        match event_handler.handle_events(
            &mut environment,
            &mut logger,
            &mut event_pump,
            &mut camera,
            &mut person_table,
            &mut editor,
            &components,
            &app_paths,
        ) {
            ControlFlow::Continue(_) => {}
            ControlFlow::Break(_) => break 'running,
        }

        renderer.begin();
        renderer.set_transformation(camera.transformation());
        render_model
            .render(&mut renderer, &environment, &editor, &logger, &person_table)
            .unwrap();

        renderer.set_transformation(Matrix::identity());

        renderer
            .draw_text(
                &format!("iteration: {}", environment.iteration()),
                (16., 16.).into(),
                16.,
                Alignment {
                    horisontal: HorisontalAlignment::Left,
                    vertical: VerticalAlignment::Top,
                },
                Color::black(),
            )
            .unwrap();

        match editor.state() {
            EditorState::Selection { selected_vessel_id } => {}
            EditorState::Placing { preset_to_place } => {
                renderer
                    .draw_text(
                        &format!("{:?}", preset_to_place),
                        event_handler.mouse_position().as_f64(),
                        16.,
                        Alignment::center(),
                        Color::black(),
                    )
                    .unwrap();
            }
        }

        if editor.about_to_delete_selected() {
            let vessel = environment
                .vessel_by_id(editor.selected_vessel().unwrap().clone())
                .unwrap();

            renderer
                .draw_text(
                    &format!(
                        "Press 'Enter' to confirm deletion of vessel: {} ({})",
                        vessel.name(),
                        vessel.id()
                    ),
                    (16., *renderer.size().y() as Float - 16.).into(),
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

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
    save_camera(camera, app_paths.camera_save_path);
}
