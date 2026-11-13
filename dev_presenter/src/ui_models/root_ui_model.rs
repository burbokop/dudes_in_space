use burbomath::camera::Camera;
use dudes_in_space_api::{environment::Environment, trade::ItemTradeTable, utils::utils::Float};
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::{
    editor::Editor,
    event_handler::EventHandler,
    logger::MemLogger,
    person_table::PersonTable,
    render::{EnvironmentRenderModel, FontProvider},
    ui_models::{
        environment_ui_model::EnvironmentUiModel, trade_table_ui_model::TradeTableUiModel,
    },
    vessel_table::VesselTable,
};

pub struct RootUiModel<'a> {
    trade_table: TradeTableUiModel<'a, 'a, 'a>,
    environment: EnvironmentUiModel<'a>,
}

impl<'a> RootUiModel<'a> {
    pub fn new(
        trade_table: &'a ItemTradeTable,
        vessel_table: &'a VesselTable,
        person_table: &'a PersonTable,
        font_provider: &'a FontProvider,
        buffer: &'a mut SharedPixelBuffer<Rgba8Pixel>,
        camera: &'a Camera<Float>,
        environment_render_model: &'a EnvironmentRenderModel,
        environment: &'a Environment,
        editor: &'a Editor,
        logger: &'a MemLogger,
        event_handler: &'a EventHandler,
    ) -> Self {
        Self {
            trade_table: TradeTableUiModel::new(trade_table, vessel_table, person_table),
            environment: EnvironmentUiModel::new(
                font_provider,
                buffer,
                camera,
                environment_render_model,
                environment,
                editor,
                logger,
                person_table,
                event_handler,
            ),
        }
    }

    pub fn apply(&mut self, req: crate::RootUiRequest) -> crate::RootUiModelSink {
        crate::RootUiModelSink {
            trade_table: self.trade_table.apply(()),
            environment: self.environment.apply(req.environment),
        }
    }
}
