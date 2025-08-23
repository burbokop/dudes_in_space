use crate::finance::{MoneyAmount, MoneyRef};
use crate::item::{ItemCount, ItemId, ItemVault, ItemVolume};
use crate::module::ModuleCapability;
use crate::trade::{BuyOffer, OfferRef, SellOffer};
use crate::utils::math::NoNeg;
use crate::utils::range::Range;
use crate::vessel::Vessel;
use std::collections::BTreeMap;

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

    pub(crate) fn cheapest_buy_offer(&self) -> Option<&OfferRef<BuyOffer>> {
        self.buy_offers.iter().min_by(|a, b| {
            a.offer
                .price_per_unit
                .amount
                .cmp(&b.offer.price_per_unit.amount)
        })
    }

    pub(crate) fn the_most_expensive_sell_offer(&self) -> Option<&OfferRef<SellOffer>> {
        self.sell_offers.iter().max_by(|a, b| {
            a.offer
                .price_per_unit
                .amount
                .cmp(&b.offer.price_per_unit.amount)
        })
    }

    pub(crate) fn eval_max_profit(
        &self,
        free_storage_space: ItemVolume,
        item_vault: &ItemVault,
    ) -> (MoneyAmount, OfferRef<BuyOffer>, OfferRef<SellOffer>) {
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
                        offer.offer.price_per_unit.clone(),
                        item_vault,
                    ),
                    offer,
                )
            })
            .min_by(|(a, _), (b, _)| a.amount.cmp(&b.amount))
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
                        offer.offer.price_per_unit.clone(),
                        item_vault,
                    ),
                    offer,
                )
            })
            .max_by(|(a, _), (b, _)| a.amount.cmp(&b.amount))
            .unwrap();

        (
            max_sell_price.amount - min_buy_price.amount,
            min_price_buy_offer.clone(),
            max_price_sell_offer.clone(),
        )
    }
}

pub(crate) struct ItemTradeTable {
    data: BTreeMap<ItemId, ItemRecord>,
}

impl ItemTradeTable {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ItemId, &ItemRecord)> {
        self.data.iter()
    }

    pub(crate) fn get(&self, id: &ItemId) -> Option<&ItemRecord> {
        self.data.get(id)
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
    price_per_unit: MoneyRef,
    item_vault: &ItemVault,
) -> MoneyRef {
    let item = item_vault.get(item_id.clone()).unwrap().upgrade().unwrap();
    let count: ItemCount = free_storage_space / item.volume;

    MoneyRef {
        currency: price_per_unit.currency,
        amount: NoNeg::wrap(count as MoneyAmount).unwrap() * price_per_unit.amount,
    }
}
