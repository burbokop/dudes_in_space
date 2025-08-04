use std::collections::VecDeque;
use std::rc::Rc;
use serde::Serialize;
use dyn_serde_macro::DeserializeSeedXXX;
use crate::item::{ItemCount};
use crate::module::ProcessTokenContext;
use crate::utils::request::{ReqFuture, ReqPromise};

#[derive(Serialize, DeserializeSeedXXX)]
pub struct ReqContext {
    find_best_buy_offer_requests: VecDeque<EnvironmentRequest<FindBestBuyOffer, FindBestBuyOfferResult>>
}

pub struct EnvironmentContext {
    process_token_context: Rc<ProcessTokenContext>,
}

impl EnvironmentContext
{
    pub fn new(
        process_token_context: Rc< ProcessTokenContext>
    ) -> Self {
    Self{
        process_token_context
    }
    }
    
    pub fn process_token_context(&self) -> &ProcessTokenContext {
        &self.process_token_context
    }
}

pub struct FindBestBuyOffer {
    pub free_storage_space: ItemCount,
}

pub struct FindBestBuyOfferResult {

}

impl FindBestBuyOffer {
    pub fn push(self, context: &mut EnvironmentContext) -> ReqFuture<FindBestBuyOfferResult> {
        context
    }
}

pub struct FindBestSellOffer {
    
}

pub struct FindBestSellOfferResult {

}

impl FindBestSellOffer {
    pub fn push(self, context: &mut EnvironmentContext) -> ReqFuture<FindBestSellOfferResult> {
        todo!()
    }
}

// pub enum EnvironmentRequestInput {
//     FindBestBuyOffer(FindBestBuyOffer),
//     FindBestSellOffer(FindBestBuyOffer),
// }

#[derive(Serialize, DeserializeSeedXXX)]
pub struct EnvironmentRequest<I, R> {
    input: I,
    promise: ReqPromise<R>
}
