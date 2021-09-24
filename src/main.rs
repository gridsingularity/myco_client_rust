use std::time::{SystemTime};
use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
struct Attributes {
    // energy_type to be specified for Offers only
    energy_type: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Requirements {
    // energy_type, energy, and price to be specified for Bids only
    trading_partners: Vec<String>,
    energy_type: Option<String>,
    energy: f64,
    price: f64,
}

trait Bid {
    // functions unique to Bids - TODO
}

trait Offer {
    // functions unique to Offers - TODO
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BaseBidOffer {
    id: String,
    time: SystemTime,
    original_price: f64,
    price: f64,
    energy: f64,
    attributes: Attributes,
    requirements: Vec<Requirements>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BidOfferMatch {
    market_id: String,
    bids: Vec<u8>,
    selected_energy: f64,
    offers: Vec<u8>,
    trade_rate: f64,
}

pub type BidOfferMatchVec = Vec<BidOfferMatch>;
pub type BaseBidOfferVec = Vec<BaseBidOffer>;

pub trait PayAsBidMatchingAlgorithm {
    fn get_matches_recommendations(&self) -> &Vec<BidOfferMatch>;   
}

impl PayAsBidMatchingAlgorithm for BidOfferMatchVec {
    // TODO
    fn get_matches_recommendations(&self) -> &Vec<BidOfferMatch> {
        &self
    }
}

pub trait FilterBaseBidOffer {
    fn filter_offers_bids_by_attribute(&self, attr_val: String) -> Vec<BaseBidOffer>;
    fn filter_offers_bids_by_requirement_trading_partner(&self, req_val: Vec<String>) -> Vec<BaseBidOffer>;
    fn filter_offers_bids_by_requirement_energy_type(&self, req_val: Option<String>) -> Vec<BaseBidOffer>;
    fn filter_offers_bids_by_requirement_energy(&self, req_val: f64) -> Vec<BaseBidOffer>;
    fn filter_offers_bids_by_requirement_price(&self, req_val: f64) -> Vec<BaseBidOffer>;
}

impl FilterBaseBidOffer for BaseBidOfferVec {
    fn filter_offers_bids_by_attribute(&self, attr_val: String) -> Vec<BaseBidOffer> {
        // Iterate over the matches to see if the attribute matches the query.
        let mut filtered = Vec::new();
        for bid_offer_match in self.clone() {
            if bid_offer_match.attributes.energy_type == Some(attr_val.clone()) {
                filtered.push(bid_offer_match);
            }
        }
        filtered
    }

    fn filter_offers_bids_by_requirement_trading_partner(&self, req_val: Vec<String>) -> Vec<BaseBidOffer> {
        // Iterate over each match's requirements to see if the trading partners matches the query.
        // TODO - how do we compare two lists of trading patners? 
        // Is there only 1 trading partner in the query and we check if it is in the list?
        let mut filtered = Vec::new();
        for bid_offer_match in self.clone() {
            let req = &bid_offer_match.requirements;
            let iter = req.into_iter().filter(|x| x.trading_partners == req_val).collect::<Vec<&Requirements>>();
            if ! iter.is_empty() {
                filtered.push(bid_offer_match);
            }
        }
        filtered
    }

    fn filter_offers_bids_by_requirement_energy_type(&self, req_val: Option<String>) -> Vec<BaseBidOffer> {
        // Iterate over each match's requirements to see if the energy type matches the query.
        let mut filtered = Vec::new();
        for bid_offer_match in self.clone() {
            let req = &bid_offer_match.requirements;
            let iter = req.into_iter().filter(|x| x.energy_type == req_val).collect::<Vec<&Requirements>>();
            if ! iter.is_empty() {
                filtered.push(bid_offer_match);
            }
        }
        filtered
    }

    fn filter_offers_bids_by_requirement_energy(&self, req_val: f64) -> Vec<BaseBidOffer> {
        // Iterate over each match's requirements to see if the energy value type matches the query.
        let mut filtered = Vec::new();
        for bid_offer_match in self.clone() {
            let req = &bid_offer_match.requirements;
            let iter = req.into_iter().filter(|x| x.energy == req_val).collect::<Vec<&Requirements>>();
            if ! iter.is_empty() {
                filtered.push(bid_offer_match);
            }
        }
        filtered
    }

    fn filter_offers_bids_by_requirement_price(&self, req_val: f64) -> Vec<BaseBidOffer> {
        // Iterate over each match's requirements to see if the price matches the query.
        let mut filtered = Vec::new();
        for bid_offer_match in self.clone() {
            let req = &bid_offer_match.requirements;
            let iter = req.into_iter().filter(|x| x.price == req_val).collect::<Vec<&Requirements>>();
            if ! iter.is_empty() {
                filtered.push(bid_offer_match);
            }
        }
        filtered
    }
}

fn main() {

}