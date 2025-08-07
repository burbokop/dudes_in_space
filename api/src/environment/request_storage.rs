use crate::item::{BuyOffer, ItemCount, Money, OfferRef, SellOffer};
use crate::utils::request::{ReqFuture, ReqPromise};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RequestStorage {
    pub(crate) find_best_buy_offer_requests:
        VecDeque<EnvironmentRequest<FindBestBuyOffer, FindBestBuyOfferResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestBuyOffer {
    pub free_storage_space: ItemCount,
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
