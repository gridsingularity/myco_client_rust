use crate::primitives::web2::{BidOfferMatch, MatchingData};
use std::collections::HashMap;
const FLOATING_POINT_TOLERANCE: f32 = 0.00001;

pub trait PayAsBid {
    fn pay_as_bid(&mut self) -> Vec<BidOfferMatch>;
}

impl PayAsBid for MatchingData {
    fn pay_as_bid(&mut self) -> Vec<BidOfferMatch> {
        let mut bid_offer_pairs = Vec::new();

        self.bids.sort_by(|a, b| b.energy_rate.partial_cmp(&a.energy_rate).unwrap());
        self.offers.sort_by(|a, b| b.energy_rate.partial_cmp(&a.energy_rate).unwrap());

        let mut available_order_energy: HashMap<String,f32> = HashMap::new();
        for offer in self.offers.clone() {
            for bid in self.bids.clone() {
                if offer.seller == bid.buyer {
                    continue;
                }

                if offer.energy_rate - bid.energy_rate > FLOATING_POINT_TOLERANCE {
                    continue;
                }

                if !available_order_energy.contains_key(bid.id.as_str()) {
                    available_order_energy.insert(bid.id.clone(), bid.energy).unwrap_or(0.0);
                }
                if !available_order_energy.contains_key(offer.id.as_str()) {
                    available_order_energy.insert(offer.id.clone(), offer.energy).unwrap_or(0.0);
                }

                let offer_energy = available_order_energy.get(
                    offer.id.as_str()).unwrap().clone();
                let bid_energy = available_order_energy.get(
                    bid.id.as_str()).unwrap().clone();

                let selected_energy = offer_energy.min(bid_energy);

                if selected_energy <= FLOATING_POINT_TOLERANCE {
                    continue;
                }

                available_order_energy.insert(bid.id.clone(), bid_energy - selected_energy);
                available_order_energy.insert(offer.id.clone(),
                                              offer_energy - selected_energy);

                assert!(available_order_energy.values().all(
                    |energy| *energy >= -FLOATING_POINT_TOLERANCE));

                let new_bid_offer_match = BidOfferMatch {
                        market_id: self.market_id.clone(),
                        time_slot: offer.time_slot,
                        bid: bid.clone(),
                        selected_energy,
                        trade_rate: bid.energy_rate,
                        offer: offer.clone(),
                };
                bid_offer_pairs.push(new_bid_offer_match);

                if let Some(offer_residual_energy) = available_order_energy.get(
                    offer.id.as_str()) {
                    if *offer_residual_energy <= FLOATING_POINT_TOLERANCE {
                        break;
                    }
                }
            }
        }
        bid_offer_pairs
    }
}