use crate::environment::{
    EnvironmentContext, FindBestBuyOfferResult, FindBestBuyVesselOfferResult,
    FindBestOffersForItemsResult, FindOwnedVesselsResult, Nebula, PlaceBuyCustomVesselOrderResult,
    RequestStorage,
};
use crate::finance::{BankRegistry, CurrencyGenerator, NotEnoughMoneyInWallet, WalletRegistry};
use crate::item::{ItemId, ItemVault};
use crate::module::{Module, ModuleCapability, ProcessTokenContext};
use crate::person::{Logger, ObjectiveDeciderVault, StatusCollector, SubordinationTable};
use crate::trade::{BuyOffer, ItemTradeTable, OfferRef, SellOffer, VesselTradeTable};
use crate::utils::request::ReqContext;
use crate::vessel::{Vessel, VesselConsole, VesselId, VesselIdPath, VesselSeed};
use dyn_serde::{DynDeserializeSeedVault, VecSeed};
use dyn_serde_macro::DeserializeSeedXXX;
use serde::Serialize;
use std::collections::BTreeMap;
use std::ops::ControlFlow;

#[derive(Debug, Serialize, DeserializeSeedXXX)]
#[deserialize_seed_xxx(seed = crate::environment::EnvironmentSeed::<'v>)]
pub struct Environment {
    #[deserialize_seed_xxx(seed = self.seed.vessel_seed)]
    vessels: Vec<Vessel>,
    nebulae: Vec<Nebula>,
    request_storage: RequestStorage,
    #[serde(default)]
    iteration: u64,
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
            iteration: 0,
        }
    }

    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    pub fn vessels(&self) -> &[Vessel] {
        &self.vessels
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
        item_vault: &ItemVault,
        subordination_table: &SubordinationTable,
        bank_registry: &BankRegistry,
        wallet_registry: &WalletRegistry,
        currency_generator: &CurrencyGenerator,
        logger: &mut dyn Logger,
    ) {
        let mut environment_context = EnvironmentContext::new(
            process_token_context,
            &mut self.request_storage,
            subordination_table,
            bank_registry,
            currency_generator,
        );
        for v in &mut self.vessels {
            v.proceed(&mut environment_context, decider_vault, logger)
        }
        self.process_requests(req_context, item_vault, bank_registry, wallet_registry);
        self.iteration += 1;
    }

    pub fn collect_status(&self, collector: &mut dyn StatusCollector) {
        collector.enter_environment(self);
        for v in &self.vessels {
            v.collect_status(collector);
        }
        collector.exit_environment();
    }

    fn process_requests(
        &mut self,
        req_context: &ReqContext,
        item_vault: &ItemVault,
        bank_registry: &BankRegistry,
        wallet_registry: &WalletRegistry,
    ) {
        self.request_storage
            .find_best_buy_offer_requests
            .retain_mut(|req| {
                assert!(req.promise.check_pending(req_context));

                let trade_table = ItemTradeTable::build(&self.vessels);
                if let Some((
                    (max_estimated_profit, max_profit_buy_offer, max_profit_sell_offer),
                    max_profit_record,
                )) = trade_table
                    .iter()
                    .map(|(item_id, record)| {
                        (
                            record.eval_max_profit(
                                bank_registry,
                                req.input.free_storage_space,
                                item_vault,
                            ),
                            record,
                        )
                    })
                    .max_by(|((a, _, _), _), ((b, _, _), _)| a.cmp(bank_registry, b))
                {
                    req.promise
                        .make_ready(
                            req_context,
                            FindBestBuyOfferResult {
                                max_estimated_profit,
                                max_profit_buy_offer,
                                max_profit_sell_offer,
                            },
                        )
                        .unwrap();
                    false
                } else {
                    true
                }
            });

        self.request_storage
            .find_best_buy_vessel_offer_requests
            .retain_mut(|req| {
                assert!(req.promise.check_pending(req_context));

                let trade_table = VesselTradeTable::build(&self.vessels);

                if let Some(offer) = trade_table
                    .offers()
                    .filter(|offer| {
                        req.input
                            .required_capabilities
                            .iter()
                            .all(|x| offer.offer.capabilities.contains(x))
                            && req
                                .input
                                .required_primary_capabilities
                                .iter()
                                .all(|x| offer.offer.primary_capabilities.contains(x))
                    })
                    .min_by(|a, b| {
                        a.offer
                            .price_per_unit
                            .cmp(bank_registry, &b.offer.price_per_unit)
                    })
                {
                    req.promise
                        .make_ready(
                            req_context,
                            FindBestBuyVesselOfferResult::BuyVesselOffer(offer.clone()),
                        )
                        .unwrap();
                    return false;
                }

                if let Some((offer, estimate)) =
                    trade_table
                        .custom_offers()
                        .filter(|(offer, _)| {
                            req.input
                                .required_capabilities
                                .iter()
                                .all(|x| offer.offer.available_capabilities.contains_key(x))
                                && req.input.required_primary_capabilities.iter().all(|x| {
                                    offer.offer.available_primary_capabilities.contains_key(x)
                                })
                        })
                        .map(|(offer, module)| {
                            assert!(
                                !req.input.required_capabilities.is_empty()
                                    || !req.input.required_primary_capabilities.is_empty()
                            );
                            (
                                offer,
                                module
                                    .trading_console()
                                    .unwrap()
                                    .estimate_buy_custom_vessel_order(
                                        req.input.required_capabilities.clone(),
                                        req.input.required_primary_capabilities.clone(),
                                        1,
                                    )
                                    .unwrap(),
                            )
                        })
                        .min_by(|(_, a), (_, b)| a.estimate.cmp(bank_registry, &b.estimate))
                {
                    req.promise
                        .make_ready(
                            req_context,
                            FindBestBuyVesselOfferResult::BuyCustomVesselOffer {
                                offer: offer.clone(),
                                estimate: estimate.clone(),
                            },
                        )
                        .unwrap();
                    return false;
                }

                req.promise
                    .make_ready(req_context, FindBestBuyVesselOfferResult::None)
                    .unwrap();
                return false;
            });

        self.request_storage
            .find_best_offers_for_items_requests
            .retain_mut(|req| {
                assert!(req.promise.check_pending(req_context));

                let mut max_profit_buy_offers: BTreeMap<ItemId, OfferRef<BuyOffer>> =
                    Default::default();
                let mut max_profit_sell_offers: BTreeMap<ItemId, OfferRef<SellOffer>> =
                    Default::default();

                for item in &req.input.items {
                    if let Some(record) = ItemTradeTable::build(&self.vessels).get(item) {
                        if let Some(o) = record.cheapest_buy_offer(bank_registry) {
                            max_profit_buy_offers.insert(item.clone(), o.clone());
                        }
                        if let Some(o) = record.the_most_expensive_sell_offer(bank_registry) {
                            max_profit_sell_offers.insert(item.clone(), o.clone());
                        }
                    }
                }

                req.promise
                    .make_ready(
                        req_context,
                        FindBestOffersForItemsResult {
                            max_profit_buy_offers,
                            max_profit_sell_offers,
                        },
                    )
                    .unwrap();
                false
            });

        self.request_storage
            .find_owned_vessels_requests
            .retain_mut(|req| {
                assert!(req.promise.check_pending(req_context));

                let mut vessels: Vec<VesselIdPath> = Default::default();

                for vessel in &self.vessels {
                    let _: ControlFlow<()> = vessel.traverse(|path, vessel| {
                        if vessel.owner() == req.input.owner
                            && (!req.input.required_empty_pilot_seat
                                || vessel.has_empty_pilot_seat())
                            && req
                                .input
                                .required_capabilities
                                .iter()
                                .all(|x| vessel.capabilities().contains(x))
                        {
                            vessels.push(path.to_owned());
                        }

                        ControlFlow::Continue(())
                    });
                }

                // if !vessels.is_empty() {
                req.promise
                    .make_ready(req_context, FindOwnedVesselsResult { vessels })
                    .unwrap();
                return false;
                // }
                //
                // true
            });

        self.request_storage
            .place_buy_custom_vessel_order_requests
            .retain_mut(|req| {
                assert!(req.promise.check_pending(req_context));

                for vessel in &self.vessels {
                    let flow: ControlFlow<()> = vessel.traverse(|path, vessel| {
                        if vessel.id() == req.input.offer.vessel_id {
                            if let Some(mut module) =
                                vessel.module_by_id_mut(req.input.offer.module_id)
                            {
                                let order = module
                                    .trading_console_mut()
                                    .unwrap()
                                    .place_buy_custom_vessel_order(
                                        &mut wallet_registry
                                            .get(&req.input.buyer_wallet)
                                            .unwrap()
                                            .upgrade()
                                            .unwrap()
                                            .borrow_mut(),
                                        req.input.needed_capabilities.clone(),
                                        req.input.needed_primary_capabilities.clone(),
                                        1,
                                    );

                                req.promise
                                    .make_ready(
                                        req_context,
                                        match order {
                                            Ok(order) => PlaceBuyCustomVesselOrderResult::Ok(order ),
                                            Err(NotEnoughMoneyInWallet) => PlaceBuyCustomVesselOrderResult::NotEnoughMoneyInWallet,
                                        }
                                        ,
                                    )
                                    .unwrap();
                                return ControlFlow::Break(());
                            }
                        }

                        ControlFlow::Continue(())
                    });

                    if flow.is_break() {
                        return false;
                    }
                }

                req.promise
                    .make_ready(req_context, PlaceBuyCustomVesselOrderResult::OfferNotFound)
                    .unwrap();
                return false;
            });

        self.request_storage
            .request_credit_limit_increase_requests
            .retain_mut(|req| {
                assert!(req.promise.check_pending(req_context));

                for vessel in &self.vessels {
                    let flow: ControlFlow<()> = vessel.traverse(|path, vessel| {
                        let mut modules: Vec<_> = vessel
                            .modules_with_capability_mut(ModuleCapability::PersonnelRoom)
                            .collect();

                        let persons: Vec<_> = modules
                            .iter_mut()
                            .map(|module| module.persons_mut().iter_mut())
                            .flatten()
                            .collect();

                        for person in persons {
                            if person.id() == req.input.recipient {
                                person
                                    .request_handler()
                                    .unwrap()
                                    .handle_request_credit_limit_increase(req);
                                return ControlFlow::Break(());
                            }
                        }

                        ControlFlow::Continue(())
                    });

                    if flow.is_break() {
                        break;
                    }
                }

                req.promise.check_pending(req_context)
            });
    }
}
