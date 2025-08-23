use crate::module::{Module, ModuleCapability};
use crate::trade::{BuyCustomVesselOffer, BuyVesselOffer, OfferRef};
use crate::vessel::Vessel;
use std::cell::Ref;

pub(crate) struct VesselTradeTable<'a> {
    offers: Vec<OfferRef<BuyVesselOffer>>,
    custom_offers: Vec<(OfferRef<BuyCustomVesselOffer>, Ref<'a, dyn Module>)>,
}

impl<'a> VesselTradeTable<'a> {
    pub fn offers(&self) -> impl Iterator<Item = &OfferRef<BuyVesselOffer>> {
        self.offers.iter()
    }

    pub fn custom_offers(
        &self,
    ) -> impl Iterator<Item = &(OfferRef<BuyCustomVesselOffer>, Ref<'a, dyn Module>)> {
        self.custom_offers.iter()
    }

    pub(crate) fn build(vessels: &'a [Vessel]) -> Self {
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
                            let console = module.trading_console().unwrap();

                            console
                                .buy_custom_vessel_offer()
                                .into_iter()
                                .map(|offer| {
                                    (
                                        OfferRef {
                                            vessel_id: vessel.id(),
                                            module_id: module.id(),
                                            offer: offer.clone(),
                                        },
                                        Ref::clone(&module),
                                    )
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
