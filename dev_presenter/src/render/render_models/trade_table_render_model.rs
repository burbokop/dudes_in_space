use crate::person_table::PersonTable;
use crate::render::scene_graph::{
    ColumnLayout, ExtColumnLayout, ExtColumnLayoutOptions, Flickable, FlickableDirection,
    GraphicsNode, RowLayout, Text,
};
use crate::render::{Alignment, Margins, Pix, Relative, RenderError, Renderer};
use crate::vessel_table::VesselTable;
use burbomath::{Rect, Size};
use dudes_in_space_api::item::ItemId;
use dudes_in_space_api::trade::{ItemRecord, ItemTradeTable, OfferRef};
use dudes_in_space_api::utils::color::Color;
use dudes_in_space_api::utils::utils::Float;

pub(crate) struct TradeTableRenderModel {}

struct DrawRecord<'a, T: sdl2::render::RenderTarget> {
    content: Box<dyn GraphicsNode<T> + 'a>,
    visible: bool,
}

impl<'a, T: sdl2::render::RenderTarget + 'a> DrawRecord<'a, T> {
    pub fn boxed<'b>(
        vessel_table: &'a VesselTable,
        item: &'b ItemId,
        record: &'b ItemRecord,
    ) -> Box<dyn GraphicsNode<T> + 'b>
    where
        'a: 'b,
    {
        Box::new(DrawRecord {
            visible: !record.buy_offers().is_empty() || !record.sell_offers().is_empty(),
            content: ExtColumnLayout::boxed(
                Pix(0.).into(),
                vec![
                    (
                        ExtColumnLayoutOptions::relative_height(0.2),
                        Box::new(Text {
                            text: format!("{}:", item),
                            color: Color::white(),
                            alignment: Alignment::left_top(),
                            font_height: Some(Pix(32.)),
                        }) as Box<dyn GraphicsNode<_>>,
                    ),
                    (
                        Default::default(),
                        RowLayout::boxed(vec![
                            ColumnLayout::boxed(
                                record
                                    .buy_offers()
                                    .iter()
                                    .map(|offer| {
                                        Box::new(Text {
                                            text: format!(
                                                "{} -> {}",
                                                vessel_table.get(&offer.vessel_id).unwrap().name,
                                                offer.offer
                                            ),
                                            color: offer_color(offer),
                                            alignment: Alignment::left_top(),
                                            font_height: Some(Pix(24.)),
                                        })
                                            as Box<dyn GraphicsNode<_>>
                                    })
                                    .collect(),
                            ),
                            ColumnLayout::boxed(
                                record
                                    .sell_offers()
                                    .iter()
                                    .map(|offer| {
                                        Box::new(Text {
                                            text: format!(
                                                "{} -> {}",
                                                vessel_table.get(&offer.vessel_id).unwrap().name,
                                                offer.offer
                                            ),
                                            color: offer_color(offer),
                                            alignment: Alignment::left_top(),
                                            font_height: Some(Pix(24.)),
                                        })
                                            as Box<dyn GraphicsNode<_>>
                                    })
                                    .collect(),
                            ),
                        ]),
                    ),
                ],
            ),
        })
    }
}

fn offer_color<T>(offer: &OfferRef<T>) -> Color {
    if offer.active {
        Color::white()
    } else {
        Color {
            a: 1.0,
            r: 0.5,
            g: 0.5,
            b: 0.5,
        }
    }
}

impl<'a, T: sdl2::render::RenderTarget> GraphicsNode<T> for DrawRecord<'a, T> {
    fn visible(&self) -> bool {
        self.visible
    }

    fn draw(&self, renderer: &mut Renderer<T>, bounding_box: Rect<Float>) {
        self.content.draw(renderer, bounding_box);

        renderer.draw_rect(bounding_box, Color::white());
    }

    fn implicit_size(&self) -> Option<Size<Float>> {
        self.content.implicit_size()
    }
}

impl TradeTableRenderModel {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn render<T: sdl2::render::RenderTarget>(
        &self,
        renderer: &mut Renderer<T>,
        trade_table: &ItemTradeTable,
        vessel_table: &VesselTable,
        bounding_box: Rect<Float>,
        person_table: &PersonTable,
        offset: Float,
    ) -> Result<(), RenderError> {
        renderer.draw_filled_rect(
            bounding_box,
            Color {
                a: 0.8,
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
        );
        renderer.draw_rect(bounding_box, Color::black());

        Flickable::new(
            FlickableDirection::Vertical,
            (0., offset).into(),
            ExtColumnLayout::boxed(
                Pix(0.).into(),
                trade_table
                    .iter()
                    .map(|(item, record)| {
                        (
                            ExtColumnLayoutOptions::margins(Margins::vertical(Relative(0.).into())),
                            DrawRecord::boxed(vessel_table, item, record),
                        )
                    })
                    .collect(),
            ),
        )
        .draw(renderer, bounding_box);

        Ok(())
    }
}
