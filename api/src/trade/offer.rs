use crate::item::{ItemCount, ItemId};
use crate::module::{ModuleCapability, ModuleId};
use crate::person::MoneyRef;
use crate::utils::range::Range;
use crate::vessel::VesselId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type OfferId = u64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: Range<ItemCount>,
    pub price_per_unit: MoneyRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyVesselOffer {
    pub id: OfferId,
    pub capabilities: BTreeSet<ModuleCapability>,
    pub primary_capabilities: BTreeSet<ModuleCapability>,
    pub price_per_unit: MoneyRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyCustomVesselOffer {
    pub id: OfferId,
    pub available_capabilities: BTreeSet<ModuleCapability>,
    pub available_primary_capabilities: BTreeSet<ModuleCapability>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: Range<ItemCount>,
    pub price_per_unit: MoneyRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferRef<Offer> {
    pub vessel_id: VesselId,
    pub module_id: ModuleId,
    pub offer: Offer,
}
