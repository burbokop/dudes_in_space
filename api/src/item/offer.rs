use crate::item::{ItemCount, ItemId, Money};
use crate::utils::range::Range;
use serde::{Deserialize, Serialize};

type OfferId = u64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: Range<ItemCount>,
    pub price_per_unit: Money,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellOffer {
    pub id: OfferId,
    pub item: ItemId,
    pub count_range: Range<ItemCount>,
    pub price_per_unit: Money,
}
