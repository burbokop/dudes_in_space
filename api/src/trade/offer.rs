use crate::finance::Money;
use crate::item::{ItemCount, ItemId};
use crate::module::{ModuleCapability, ModuleId};
use crate::utils::non_nil_uuid::NonNilUuid;
use crate::vessel::VesselId;
use burbomath::range::RangeInclusive;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

pub type OfferId = NonNilUuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: RangeInclusive<ItemCount>,
    pub price_per_unit: Money,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyVesselOffer {
    pub id: OfferId,
    pub capabilities: BTreeSet<ModuleCapability>,
    pub primary_capabilities: BTreeSet<ModuleCapability>,
    pub price_per_unit: Money,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyCustomVesselOffer {
    pub id: OfferId,
    pub available_capabilities: BTreeMap<ModuleCapability, Money>,
    pub available_primary_capabilities: BTreeMap<ModuleCapability, Money>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: RangeInclusive<ItemCount>,
    pub price_per_unit: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferRef<Offer> {
    pub vessel_id: VesselId,
    pub module_id: ModuleId,
    pub offer: Offer,
    /// Offer is active if there is enough money in the operational wallet for this offer.
    #[serde(default)]
    pub active: bool,
}

impl Display for BuyOffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} for {}",
            self.count_range, self.item, self.price_per_unit
        )
    }
}

impl Display for SellOffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} for {}",
            self.count_range, self.item, self.price_per_unit
        )
    }
}

impl Display for BuyCustomVesselOffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}\n{:?}",
            self.available_capabilities, self.available_primary_capabilities
        )
    }
}

impl<T: Display> Display for OfferRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.({})", self.vessel_id, self.module_id, self.offer)
    }
}
