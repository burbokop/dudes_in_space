use crate::item::ItemId;
use crate::trade::{BuyCustomVesselOffer, BuyVesselOffer, OfferRef};
use crate::vessel::Vessel;
use std::collections::BTreeMap;

pub(crate) struct VesselTradeTable {
    ready_made: BTreeMap<ItemId, OfferRef<BuyVesselOffer>>,
    custom: BTreeMap<ItemId, OfferRef<BuyCustomVesselOffer>>,
}

impl VesselTradeTable {
    pub(crate) fn build(vessels: &[Vessel]) -> Self {
        todo!()
    }
}
