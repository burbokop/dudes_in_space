use dudes_in_space_api::{finance::Money, trade::ItemTradeTable};
use slint::{ModelRc, ToSharedString};

use crate::{MoneyUiModelSink, person_table::PersonTable, vessel_table::VesselTable};

pub struct TradeTableUiModel<'i, 'v, 'p> {
    trade_table: &'i ItemTradeTable,
    vessel_table: &'v VesselTable,
    person_table: &'p PersonTable,
}

impl<'i, 'v, 'p> TradeTableUiModel<'i, 'v, 'p> {
    pub fn new(
        trade_table: &'i ItemTradeTable,
        vessel_table: &'v VesselTable,
        person_table: &'p PersonTable,
    ) -> Self {
        Self {
            trade_table,
            vessel_table,
            person_table,
        }
    }

    pub fn apply(&self, req: ()) -> crate::TradeTableUiModelSink {
        fn money_to_sink(m: Money) -> MoneyUiModelSink {
            MoneyUiModelSink {
                amount: m.amount.into_inner() as i32,
                currency: m.currency.into(),
            }
        }

        let x = ModelRc::new(
            self.trade_table
                .iter()
                .map(|(item_id, record)| crate::ItemRecordUiModelSink {
                    item_id: item_id.into(),
                    buy_offers: ModelRc::new(
                        record
                            .buy_offers()
                            .into_iter()
                            .map(|offer| crate::OfferUiModelSink {
                                max_count: offer.offer.count_range.end as i32,
                                min_count: offer.offer.count_range.start as i32,
                                vessel_name: self
                                    .vessel_table
                                    .get(&offer.vessel_id)
                                    .unwrap()
                                    .name
                                    .to_shared_string(),
                                price_per_unit: money_to_sink(offer.offer.price_per_unit.clone()),
                                active: offer.active,
                            })
                            .collect::<slint::VecModel<_>>(),
                    ),
                    sell_offers: ModelRc::new(
                        record
                            .sell_offers()
                            .into_iter()
                            .map(|offer| crate::OfferUiModelSink {
                                max_count: offer.offer.count_range.end as i32,
                                min_count: offer.offer.count_range.start as i32,
                                vessel_name: self
                                    .vessel_table
                                    .get(&offer.vessel_id)
                                    .unwrap()
                                    .name
                                    .to_shared_string(),
                                price_per_unit: money_to_sink(offer.offer.price_per_unit.clone()),
                                active: offer.active,
                            })
                            .collect::<slint::VecModel<_>>(),
                    ),
                })
                .collect::<slint::VecModel<_>>(),
        );

        crate::TradeTableUiModelSink { records: x }
    }
}
