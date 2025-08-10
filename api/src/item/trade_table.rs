use crate::item::{BuyOffer, ItemCount, ItemId, ItemVault, ItemVolume, Money, SellOffer};
use crate::module::{ModuleCapability, ModuleId};
use crate::utils::range::Range;
use crate::vessel::{Vessel, VesselId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferRef<Offer> {
    vessel_id: VesselId,
    module_id: ModuleId,
    offer: Offer,
}

pub(crate) struct ItemRecord {
    id: ItemId,
    buy_offers: Vec<OfferRef<BuyOffer>>,
    sell_offers: Vec<OfferRef<SellOffer>>,
}

impl ItemRecord {
    fn new(id: ItemId) -> Self {
        Self {
            id,
            buy_offers: Default::default(),
            sell_offers: Default::default(),
        }
    }

    pub(crate) fn eval_max_profit(
        &self,
        free_storage_space: ItemVolume,
        item_vault: &ItemVault,
    ) -> (Money, OfferRef<BuyOffer>, OfferRef<SellOffer>) {
        let (min_buy_price, min_price_buy_offer) = self
            .buy_offers
            .iter()
            .filter(|offer| {
                volume_range(&offer.offer.item, offer.offer.count_range, item_vault)
                    .contains(&free_storage_space)
            })
            .map(|offer| {
                (
                    total_price(
                        &offer.offer.item,
                        free_storage_space,
                        offer.offer.price_per_unit,
                        item_vault,
                    ),
                    offer,
                )
            })
            .min_by(|(a, _), (b, _)| a.cmp(b))
            .unwrap();

        let (max_sell_price, max_price_sell_offer) = self
            .sell_offers
            .iter()
            .filter(|offer| {
                volume_range(&offer.offer.item, offer.offer.count_range, item_vault)
                    .contains(&free_storage_space)
            })
            .map(|offer| {
                (
                    total_price(
                        &offer.offer.item,
                        free_storage_space,
                        offer.offer.price_per_unit,
                        item_vault,
                    ),
                    offer,
                )
            })
            .max_by(|(a, _), (b, _)| a.cmp(b))
            .unwrap();

        (
            max_sell_price - min_buy_price,
            min_price_buy_offer.clone(),
            max_price_sell_offer.clone(),
        )
    }
}

pub(crate) struct TradeTable {
    data: BTreeMap<ItemId, ItemRecord>,
}

impl TradeTable {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ItemId, &ItemRecord)> {
        self.data.iter()
    }

    pub(crate) fn build(vessels: &[Vessel]) -> Self {
        let buy_offer_refs: Vec<_> = vessels
            .iter()
            .map(|vessel| {
                vessel
                    .modules_with_capability(ModuleCapability::TradingTerminal)
                    .map(|module| {
                        module
                            .trading_console()
                            .unwrap()
                            .buy_offers()
                            .iter()
                            .map(|offer| OfferRef {
                                vessel_id: vessel.id(),
                                module_id: module.id(),
                                offer: offer.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
            })
            .flatten()
            .flatten()
            .collect();

        let sell_offer_refs: Vec<_> = vessels
            .iter()
            .map(|vessel| {
                vessel
                    .modules_with_capability(ModuleCapability::TradingTerminal)
                    .map(|module| {
                        module
                            .trading_console()
                            .unwrap()
                            .sell_offers()
                            .iter()
                            .map(|offer| OfferRef {
                                vessel_id: vessel.id(),
                                module_id: module.id(),
                                offer: offer.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
            })
            .flatten()
            .flatten()
            .collect();

        let mut data: BTreeMap<ItemId, ItemRecord> = Default::default();

        for offer_ref in buy_offer_refs {
            data.entry(offer_ref.offer.item.clone())
                .or_insert(ItemRecord::new(offer_ref.offer.item.clone()))
                .buy_offers
                .push(offer_ref.clone());
        }

        for offer_ref in sell_offer_refs {
            data.entry(offer_ref.offer.item.clone())
                .or_insert(ItemRecord::new(offer_ref.offer.item.clone()))
                .sell_offers
                .push(offer_ref.clone());
        }

        Self { data }
    }
}

fn volume_range(
    item_id: &ItemId,
    count_range: Range<ItemCount>,
    item_vault: &ItemVault,
) -> Range<ItemVolume> {
    let item = item_vault.get(item_id.clone()).unwrap().upgrade().unwrap();
    Range {
        start: item.volume * count_range.start,
        end: item.volume * count_range.end,
    }
}

fn total_price(
    item_id: &ItemId,
    free_storage_space: ItemVolume,
    price_per_unit: Money,
    item_vault: &ItemVault,
) -> Money {
    let item = item_vault.get(item_id.clone()).unwrap().upgrade().unwrap();
    let count: ItemCount = free_storage_space / item.volume;

    count * price_per_unit
}
