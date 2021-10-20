use serde::{Serialize, Deserialize};
use chrono::{Utc, TimeZone, DateTime};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Attributes {
    // energy_type to be specified for Offers only
    energy_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Requirements {
    // energy_type, energy, and price to be specified for Bids only
    trading_partners: Vec<String>,
    energy_type: Option<String>,
    energy: f64,
    price: f64,
}

trait BidFunctions {
    // functions unique to Bids - TODO
}

trait OfferFunctions {
    // functions unique to Offers - TODO
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaseBidOffer {
    id: String,
    time: DateTime<Utc>,
    original_price: f64,
    price: f64,
    energy: f64,
    attributes: Attributes,
    requirements: Vec<Requirements>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    buyer: String,
    energy_rate: f64,
    energy: f64,
    id: i8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    seller: String,
    energy_rate: f64,
    energy: f64,
    id: i8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BidOfferMatch {
    market_id: String,
    bids: Vec<Bid>,
    selected_energy: f64,
    offers: Vec<Offer>,
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

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn filter_by_energy_works() {
        let requirements_a = Requirements {
            trading_partners: vec!["Charlie".to_string(), "Mike".to_string(), "Victor".to_string()],
            energy_type: Some("green".to_string()),
            energy: 25.0,
            price: 15.0,
        };
    
        let requirements_b = Requirements {
            trading_partners: vec!["Charlie".to_string(), "Mike".to_string(), "Victor".to_string()],
            energy_type: Some("coal".to_string()),
            energy: 40.0,
            price: 14.0,
        };

        let base_bid_offer_a = BaseBidOffer {
            id: "a".to_string(),
            time: Utc.timestamp(100200, 0),
            original_price: 12.0,
            price: 14.0,
            energy: 25.0,
            attributes: Attributes { energy_type: Some("coal".to_string()) },
            requirements: vec![requirements_a],
        };

        let base_bid_offer_b = BaseBidOffer {
            id: "b".to_string(),
            time: Utc.timestamp(100500, 0),
            original_price: 11.0,
            price: 15.0,
            energy: 40.0,
            attributes: Attributes { energy_type: Some("green".to_string()) },
            requirements: vec![requirements_b],
        };

        let mut base_bid_offer_vec = Vec::new();
        base_bid_offer_vec.push(base_bid_offer_a.clone());
        base_bid_offer_vec.push(base_bid_offer_b);
        let mut expected = Vec::new();
        expected.push(base_bid_offer_a);
        
        assert_eq!(
            base_bid_offer_vec.filter_offers_bids_by_requirement_energy(25.0), 
            expected);
        assert_ne!(
                base_bid_offer_vec.filter_offers_bids_by_requirement_energy(40.0), 
                expected);
            
    }

    #[test]
    fn filter_by_energy_type_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn filter_by_trading_partner_works() {
        let requirements_a = Requirements {
            trading_partners: vec!["Charlie".to_string(), "Mike".to_string(), "Victor".to_string()],
            energy_type: Some("green".to_string()),
            energy: 25.0,
            price: 15.0,
        };
    
        let requirements_b = Requirements {
            trading_partners: vec!["Charlie".to_string(), "Mike".to_string(), "Victor".to_string()],
            energy_type: Some("coal".to_string()),
            energy: 40.0,
            price: 14.0,
        };
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn filter_by_price_works() {
        assert_eq!(2 + 2, 4);
    }
}