use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::item::ItemCount;
use crate::utils::request::{ReqFuture, ReqPromise};

#[derive(Debug, Serialize, Deserialize, Default)]
// #[deserialize_seed_xxx(seed=crate::environment::request_storage::RequestStorageSeed)]
pub struct RequestStorage {
    find_best_buy_offer_requests: VecDeque<EnvironmentRequest<FindBestBuyOffer, FindBestBuyOfferResult>>
}

pub(crate) struct RequestStorageSeed {

}

impl RequestStorageSeed {
    pub(crate) fn new() -> Self {
        Self {}
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestBuyOffer {
    pub free_storage_space: ItemCount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestBuyOfferResult {

}

impl FindBestBuyOffer {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestBuyOfferResult> {
        let (promise, future) = ReqPromise::new();
        context.find_best_buy_offer_requests.push_back(EnvironmentRequest { promise, input: self });
        future
    }
}

#[derive(Serialize, Deserialize)]
pub struct FindBestSellOffer {

}

#[derive(Serialize, Deserialize)]
pub struct FindBestSellOfferResult {

}

impl FindBestSellOffer {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestSellOfferResult> {
        todo!()
    }
}

// pub enum EnvironmentRequestInput {
//     FindBestBuyOffer(FindBestBuyOffer),
//     FindBestSellOffer(FindBestBuyOffer),
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentRequest<I, R> {
    input: I,
    promise: ReqPromise<R>
}
//
// pub(crate) EnvironmentRequestSeed {
//
// }