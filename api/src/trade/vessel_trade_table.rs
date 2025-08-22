use crate::module::ModuleCapability;
use crate::trade::{BuyCustomVesselOffer, BuyVesselOffer, OfferRef};
use crate::vessel::Vessel;

pub(crate) struct VesselTradeTable {
    offers: Vec<OfferRef<BuyVesselOffer>>,
    custom_offers: Vec<OfferRef<BuyCustomVesselOffer>>,
}

impl VesselTradeTable {
    pub fn offers(&self) -> impl Iterator<Item = OfferRef<BuyVesselOffer>> {
        self.offers.iter()
    }

    pub fn custom_offers(&self) -> impl Iterator<Item = OfferRef<BuyCustomVesselOffer>> {
        self.custom_offers.iter()
    }

    pub(crate) fn build(vessels: &[Vessel]) -> Self {
        Self {
            offers: vessels
                .iter()
                .map(|vessel| {
                    vessel
                        .modules_with_capability(ModuleCapability::VesselSellingTerminal)
                        .map(|module| {
                            module
                                .trading_console()
                                .unwrap()
                                .buy_vessel_offers()
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
                .collect(),
            custom_offers: vessels
                .iter()
                .map(|vessel| {
                    vessel
                        .modules_with_capability(ModuleCapability::VesselSellingTerminal)
                        .map(|module| {
                            module
                                .trading_console()
                                .unwrap()
                                .buy_custom_vessel_offer()
                                .into_iter()
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
                .collect(),
        }
    }
}
