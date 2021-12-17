use serde::{Serialize, Deserialize};
use chrono::{NaiveDateTime};

const FLOATING_POINT_TOLERANCE: f32 = 0.00001;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Bid {
    pub r#type: String,
    pub id: String,
    pub energy: f32,
    pub energy_rate: f32,
    pub original_price: f32,
    pub attributes: Option<String>,
    pub requirements: Option<String>,
    pub buyer_origin: String,
    pub buyer_origin_id: String,
    pub buyer_id: String,
    pub buyer: String,
    pub time_slot: Option<NaiveDateTime>,
    pub creation_time: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Offer {
    pub r#type: String,
    pub id: String,
    pub energy: f32,
    pub energy_rate: f32,
    pub original_price: f32,
    pub attributes: Option<String>,
    pub requirements: Option<String>,
    pub seller_origin: String,
    pub seller_origin_id: String,
    pub seller_id: String,
    pub seller: String,
    pub time_slot: Option<NaiveDateTime>,
    pub creation_time: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BidOfferMatch {
    market_id: String,
    bids: Vec<Bid>,
    selected_energy: f32,
    offers: Vec<Offer>,
    trade_rate: f32,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MatchingData {
    pub bids: Vec<Bid>,
    pub offers: Vec<Offer>,
}

pub trait GetMatchesRecommendations {
    fn get_matches_recommendations(&self, mut matching_data: MatchingData) -> Vec<BidOfferMatch> {
        let mut bid_offer_pairs = Vec::new();

        matching_data.bids.sort_by(|a, b| b.energy_rate.partial_cmp(&a.energy_rate).unwrap());
        matching_data.offers.sort_by(|a, b| b.energy_rate.partial_cmp(&a.energy_rate).unwrap());

        let mut already_selected_bids = Vec::new();
        for offer in matching_data.offers.clone() {
            for bid in matching_data.bids.clone() {
                if already_selected_bids.contains(&bid.id) || offer.seller == bid.buyer {
                    continue;
                }
                if offer.energy_rate - bid.energy_rate <= FLOATING_POINT_TOLERANCE {
                    already_selected_bids.push(bid.id.clone());
                    let selected_energy = bid.energy.min(offer.energy);
                    let new_bid_offer_match = BidOfferMatch {
                            market_id: bid.id.clone(),
                            bids: vec![bid.clone()],
                            selected_energy: selected_energy,
                            offers: vec![offer.clone()],
                            trade_rate: bid.energy_rate,
                    };
                    bid_offer_pairs.push(new_bid_offer_match);
                    break;
                }
            }
        }
        bid_offer_pairs
    }
}

impl GetMatchesRecommendations for MatchingData {

}