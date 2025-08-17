use crate::item::{ItemCount, ItemId};
use crate::module::ModuleCapability;
use crate::person::MoneyRef;
use crate::utils::range::Range;
use serde::{Deserialize, Serialize};

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
    pub primary_caps: Vec<ModuleCapability>,
    pub price_per_unit: MoneyRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: Range<ItemCount>,
    pub price_per_unit: MoneyRef,
}
