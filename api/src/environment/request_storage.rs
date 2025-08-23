use crate::finance::MoneyAmount;
use crate::item::{ItemId, ItemVolume};
use crate::module::ModuleCapability;
use crate::person::PersonId;
use crate::trade::{BuyCustomVesselOffer, BuyOffer, BuyVesselOffer, OfferRef, SellOffer};
use crate::utils::request::{ReqFuture, ReqPromise};
use crate::vessel::{VesselId, VesselIdPath};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RequestStorage {
    pub(crate) find_best_buy_offer_requests:
        VecDeque<EnvironmentRequest<FindBestBuyOffer, FindBestBuyOfferResult>>,

    pub(crate) find_best_buy_vessel_offer_requests:
        VecDeque<EnvironmentRequest<FindBestBuyVesselOffer, FindBestBuyVesselOfferResult>>,

    pub(crate) find_best_offers_for_items_requests:
        VecDeque<EnvironmentRequest<FindBestOffersForItems, FindBestOffersForItemsResult>>,

    pub(crate) find_owned_vessels:
        VecDeque<EnvironmentRequest<FindOwnedVessels, FindOwnedVesselsResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestOffersForItems {
    pub items: BTreeSet<ItemId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestOffersForItemsResult {
    pub max_profit_buy_offers: BTreeMap<ItemId, OfferRef<BuyOffer>>,
    pub max_profit_sell_offers: BTreeMap<ItemId, OfferRef<SellOffer>>,
}

impl FindBestOffersForItems {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestOffersForItemsResult> {
        let (promise, future) = ReqPromise::new();
        context
            .find_best_offers_for_items_requests
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
    pub max_estimated_profit: MoneyAmount,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestBuyVesselOffer {
    #[serde(with = "crate::utils::tagged_option")]
    pub prefer_to_buy_from: Option<VesselId>,
    pub required_capabilities: BTreeSet<ModuleCapability>,
    pub required_primary_capabilities: BTreeSet<ModuleCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "offer_type")]
pub enum FindBestBuyVesselOfferResult {
    BuyVesselOffer(OfferRef<BuyVesselOffer>),
    BuyCustomVesselOffer(OfferRef<BuyCustomVesselOffer>),
    None,
}

impl FindBestBuyVesselOffer {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestBuyVesselOfferResult> {
        let (promise, future) = ReqPromise::new();
        context
            .find_best_buy_vessel_offer_requests
            .push_back(EnvironmentRequest {
                promise,
                input: self,
            });
        future
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestSellOffer {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindBestSellOfferResult {}

impl FindBestSellOffer {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindBestSellOfferResult> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindOwnedVessels {
    pub owner: PersonId,
    pub required_capabilities: BTreeSet<ModuleCapability>,
    pub required_empty_pilot_seat: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindOwnedVesselsResult {
    pub vessels: Vec<VesselIdPath>,
}

impl FindOwnedVessels {
    pub fn push(self, context: &mut RequestStorage) -> ReqFuture<FindOwnedVesselsResult> {
        let (promise, future) = ReqPromise::new();
        context.find_owned_vessels.push_back(EnvironmentRequest {
            promise,
            input: self,
        });
        future
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentRequest<I, R> {
    pub(crate) input: I,
    pub(crate) promise: ReqPromise<R>,
}
