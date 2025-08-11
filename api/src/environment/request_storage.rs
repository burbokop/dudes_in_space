use crate::item::{BuyOffer, ItemId, ItemVolume, Money, OfferRef, SellOffer};
use crate::utils::request::{ReqFuture, ReqPromise};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RequestStorage {
    pub(crate) find_best_buy_offer_requests:
        VecDeque<EnvironmentRequest<FindBestBuyOffer, FindBestBuyOfferResult>>,

    pub(crate) find_best_offers_for_item_requests:
        VecDeque<EnvironmentRequest<FindBestOffersForItem, FindBestOffersForItemResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestOffersForItem {
    pub item: ItemId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestOffersForItemResult {
    #[serde(with = "crate::utils::tagged_option")]
    pub max_profit_buy_offer: Option<OfferRef<BuyOffer>>,
    #[serde(with = "crate::utils::tagged_option")]
    pub max_profit_sell_offer: Option<OfferRef<SellOffer>>,
}

impl FindBestOffersForItem {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestOffersForItemResult> {
        let (promise, future) = ReqPromise::new();
        context
            .find_best_offers_for_item_requests
            .push_back(EnvironmentRequest {
                promise,
                input: self,
            });
        future
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestBuyOffer {
    pub free_storage_space: ItemVolume,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestBuyOfferResult {
    pub max_estimated_profit: Money,
    pub max_profit_buy_offer: OfferRef<BuyOffer>,
    pub max_profit_sell_offer: OfferRef<SellOffer>,
}

impl FindBestBuyOffer {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestBuyOfferResult> {
        let (promise, future) = ReqPromise::new();
        context
            .find_best_buy_offer_requests
            .push_back(EnvironmentRequest {
                promise,
                input: self,
            });
        future
    }
}

#[derive(Serialize, Deserialize)]
pub struct FindBestSellOffer {}

#[derive(Serialize, Deserialize)]
pub struct FindBestSellOfferResult {}

impl FindBestSellOffer {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestSellOfferResult> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentRequest<I, R> {
    pub(crate) input: I,
    pub(crate) promise: ReqPromise<R>,
}
