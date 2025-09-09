use dudes_in_space_api::environment::EnvironmentContext;
use dudes_in_space_api::finance::Money;
use dudes_in_space_api::item::{ItemCount, ItemId};
use dudes_in_space_api::module::{ModuleConsole, ModuleId};
use dudes_in_space_api::person::{Objective, ObjectiveStatus, PersonLogger, ThisPerson};
use dudes_in_space_api::utils::range::Range;
use dudes_in_space_api::vessel::VesselInternalConsole;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
// #[derive(Debug, Serialize, DeserializeSeedXXX)]
#[serde(tag = "place_buy_custom_vessel_offer_objective_stage")]
// #[deserialize_seed_xxx(seed = crate::objectives::management::place_buy_custom_vessel_offer_objective::PlaceBuyCustomVesselOfferObjectiveSeed::<'context>)]
pub(crate) enum PlaceSellOffersObjective {
    FindTerminal {
        offers: BTreeMap<ItemId, (Range<ItemCount>, Money)>,
    },
    MoveToTerminal {
        dst: ModuleId,
        offers: BTreeMap<ItemId, (Range<ItemCount>, Money)>,
    },
    PlaceOffers {
        offers: BTreeMap<ItemId, (Range<ItemCount>, Money)>,
    },
}

struct PlaceSellOfferObjectiveSeed {}

impl PlaceSellOffersObjective {
    pub fn new(offers: BTreeMap<ItemId, (Range<ItemCount>, Money)>) -> Self {
        todo!()
    }
}

impl Objective for PlaceSellOffersObjective {
    type Error = PlaceSellOfferObjectiveError;

    fn pursue(
        &mut self,
        this_person: &mut ThisPerson,
        this_module: &mut dyn ModuleConsole,
        this_vessel: &dyn VesselInternalConsole,
        environment_context: &mut EnvironmentContext,
        logger: &mut PersonLogger,
    ) -> Result<ObjectiveStatus, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum PlaceSellOfferObjectiveError {}

impl Display for PlaceSellOfferObjectiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PlaceSellOfferObjectiveError {}
