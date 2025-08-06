use crate::environment::{EnvironmentContext, FindBestBuyOfferResult, Nebula, RequestStorage};
use crate::item::{ TradeTable};
use crate::module::{Module,  ProcessTokenContext};
use crate::person::{Logger, ObjectiveDeciderVault};
use crate::vessel::{Vessel, VesselId, VesselSeed};
use dyn_serde::{DynDeserializeSeedVault, VecSeed};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use crate::utils::request::ReqContext;

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::environment::EnvironmentSeed::<'v>)]
pub struct Environment {
    #[deserialize_seed_xxx(seed = self.seed.vessel_seed)]
    vessels: Vec<Vessel>,
    nebulae: Vec<Nebula>,
    request_storage: RequestStorage,
}

pub struct EnvironmentSeed<'v> {
    vessel_seed: VecSeed<VesselSeed<'v>>,
}

impl<'v> EnvironmentSeed<'v> {
    pub fn new(vault: &'v DynDeserializeSeedVault<dyn Module>) -> Self {
        Self {
            vessel_seed: VecSeed::new(VesselSeed::new(vault)),
        }
    }
}

impl Environment {
    pub fn new(vessels: Vec<Vessel>, nebulae: Vec<Nebula>) -> Self {
        Self {
            vessels,
            nebulae,
            request_storage: Default::default(),
        }
    }

    pub(crate) fn vessel_by_id(&self, id: VesselId) -> Option<&Vessel> {
        self.vessels.iter().find(|v| v.id() == id)
    }

    pub fn vessel_by_id_mut(&mut self, id: VesselId) -> Option<&mut Vessel> {
        self.vessels.iter_mut().find(|v| v.id() == id)
    }

    pub fn proceed(
        &mut self,
        process_token_context: &ProcessTokenContext,
        req_context: &ReqContext,
        decider_vault: &ObjectiveDeciderVault,
        logger: &mut dyn Logger,
    ) {
        let mut environment_context =
            EnvironmentContext::new(process_token_context, &mut self.request_storage);
        for v in &mut self.vessels {
            v.proceed(&mut environment_context, decider_vault, logger)
        }
        self.process_requests(req_context);
    }

    fn process_requests(&mut self, req_context: & ReqContext) {
        for req in &mut self.request_storage.find_best_buy_offer_requests {


            let trade_table = TradeTable::build(&self.vessels);

            let ((max_estimated_profit, max_profit_buy_offer, max_profit_sell_offer), max_profit_record) = trade_table
                .iter()
                .map(|(item_id, record)| {
                    (record.eval_max_profit(req.input.free_storage_space), record)
                })
                .max_by(|((a,_,_), _), ((b,_,_), _)| a.cmp(b))
                .unwrap();

            req.promise.make_ready(req_context, FindBestBuyOfferResult {
                max_estimated_profit,
                max_profit_buy_offer,
                max_profit_sell_offer
            } ).unwrap()
        }
    }
}
