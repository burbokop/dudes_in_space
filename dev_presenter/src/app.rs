use std::{cell::RefCell, ops::ControlFlow, path::PathBuf, rc::Rc};

use burbomath::camera::Camera;
use dudes_in_space_api::{environment::Environment, trade::ItemTradeTable, utils::utils::Float};
use dudes_in_space_core::{
    components::{Components, core_components},
    module_types,
};

use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::{
    editor::Editor,
    event_handler::{Event, EventHandler},
    logger::MemLogger,
    person_table::PersonTable,
    render::{
        EnvironmentRenderModel, FontProvider, ModuleTextureContainerBuilder, TradeTableRenderModel,
    },
    ui_models::root_ui_model::RootUiModel,
    utils::{load, load_camera, load_logger},
    vessel_table::VesselTable,
};

pub(crate) struct SavePaths {
    pub(crate) save_path: PathBuf,
    pub(crate) camera_save_path: PathBuf,
    pub(crate) logger_save_path: PathBuf,
}

pub(crate) struct App {
    pub(crate) save_paths: SavePaths,
    event_handler: EventHandler,
    environment: Environment,
    trade_table: ItemTradeTable,
    logger: MemLogger,
    pub(crate) camera: Camera<Float>,
    person_table: PersonTable,
    vessel_table: VesselTable,
    editor: Editor,
    components: Components,
    font_provider: FontProvider,
    environment_render_model: EnvironmentRenderModel,
    buffer: SharedPixelBuffer<Rgba8Pixel>,
    pub(crate) desired_tps: Float,
    pub(crate) tps: Float,
    // canvas: Canvas<Window>,
    // event_pump: EventPump,
}

impl App {
    pub(crate) fn new() -> Rc<RefCell<Self>> {
        let save_paths = SavePaths {
            save_path: std::env::home_dir()
                .unwrap()
                .join(".dudes_in_space/save.json"),
            camera_save_path: std::env::home_dir()
                .unwrap()
                .join(".dudes_in_space/camera.json"),
            logger_save_path: std::env::home_dir()
                .unwrap()
                .join(".dudes_in_space/logger.json"),
        };

        // let sdl_context = sdl2::init().unwrap();

        // let video_subsystem = sdl_context.video().unwrap();

        // video_subsystem.into;

        // let window = video_subsystem
        //     .window("rust-sdl2 demo: Video", 1600, 800)
        //     .position_centered()
        //     .resizable()
        //     .opengl()
        //     .build()
        //     .unwrap();

        // let canvas = window.into_canvas().build().unwrap();

        let camera = load_camera(save_paths.camera_save_path.clone());
        let logger = load_logger(save_paths.logger_save_path.clone());

        let module_bg_tex_container = ModuleTextureContainerBuilder::new()
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
            .with(
                module_types::TRADING_TERMINAL.into(),
                include_bytes!("../assets/trading_terminal.png"),
            )
            .with(
                module_types::VESSEL_SELLING_TERMINAL.into(),
                include_bytes!("../assets/vessel_selling_terminal.png"),
            )
            .with(
                module_types::PLANT_FACILITY.into(),
                include_bytes!("../assets/plant_facility.png"),
            )
            .with_aseprite_spritesheet(
                module_types::ORE_MANIFOLD.into(),
                include_bytes!("../assets/ore_manifold.png"),
                include_bytes!("../assets/ore_manifold.json"),
            )
            .build();

        let environment_render_model = EnvironmentRenderModel::new(module_bg_tex_container);
        let trade_table_render_model = TradeTableRenderModel::new();
        let font_provider = FontProvider::new();
        let components = core_components();
        let environment = load(&components, save_paths.save_path.clone());
        let person_table = PersonTable::new(&environment);
        let vessel_table = VesselTable::new(&environment);

        // let event_pump = sdl_context.event_pump().unwrap();
        let event_handler = EventHandler::new();
        let editor = Editor::new();
        let trade_table = ItemTradeTable::build(
            &components.bank_registry,
            &components.wallet_registry,
            environment.vessels(),
        );

        Rc::new(RefCell::new(Self {
            save_paths,
            event_handler,
            environment,
            trade_table,
            logger,
            camera,
            person_table,
            vessel_table,
            editor,
            components,
            font_provider,
            environment_render_model,
            desired_tps: 30.,
            buffer: SharedPixelBuffer::new(0, 0),
            tps: 1.,
            // canvas,
            // event_pump,
        }))
    }

    pub(crate) fn handle_event(&mut self, event: Event) -> ControlFlow<()> {
        self.event_handler.handle_event(
            &mut self.environment,
            &mut self.trade_table,
            &mut self.logger,
            &mut self.camera,
            &mut self.person_table,
            &mut self.vessel_table,
            &mut self.editor,
            &self.components,
            &self.save_paths,
            event,
        )
    }

    pub(crate) fn root_ui_model<'a>(&'a mut self) -> RootUiModel<'a> {
        RootUiModel::new(
            &self.trade_table,
            &self.vessel_table,
            &self.person_table,
            &self.font_provider,
            &mut self.buffer,
            &self.camera,
            &self.environment_render_model,
            &self.environment,
            &self.editor,
            &self.logger,
            &self.event_handler,
        )
    }

    pub(crate) fn proceed(&mut self) -> ControlFlow<()> {
        // for event in self.event_pump.poll_iter() {
        //     match event {
        //         sdl2::event::Event::Quit { timestamp } => return ControlFlow::Break(()),
        //         sdl2::event::Event::MouseWheel {
        //             timestamp,
        //             window_id,
        //             which,
        //             x,
        //             y,
        //             direction,
        //             precise_x,
        //             precise_y,
        //             mouse_x,
        //             mouse_y,
        //         } => println!("{}, {}", x, y),
        //         _ => {}
        //     }
        // }

        // self.canvas.present();

        ControlFlow::Continue(())
    }
}
