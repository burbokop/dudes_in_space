use crate::module::ProcessTokenContext;
use crate::utils::request::ReqFuture;

pub struct EnvironmentContext {
    
}

impl EnvironmentContext {
    pub fn new() -> Self {
    Self{}
    }
    
    pub fn process_token_context(&self) -> &ProcessTokenContext {
        todo!()
    }
}

pub struct FindBestBuyOffer {
    
}

pub struct FindBestBuyOfferResult {

}

impl FindBestBuyOffer {
    pub fn push(self, context: &mut EnvironmentContext) -> ReqFuture<FindBestBuyOfferResult> {
        todo!()
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

pub enum EnvironmentRequest {
    FindBestBuyOffer(FindBestBuyOffer),
    FindBestSellOffer(FindBestBuyOffer),
}
