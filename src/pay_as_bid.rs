use serde::{Serialize, Deserialize};

const FLOATING_POINT_TOLERANCE: f32 = 0.00001;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Bid {
    buyer: String,
    energy_rate: f32,
    energy: f32,
    id: u8,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Offer {
    seller: String,
    energy_rate: f32,
    energy: f32,
    id: u8,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BidOfferMatch {
    market_id: u8,
    bids: Vec<Bid>,
    selected_energy: f32,
    offers: Vec<Offer>,
    trade_rate: f32,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MatchingData {
    bids: Vec<Bid>,
    offers: Vec<Offer>,
}

pub trait GetMatchesRecommendations {
    fn get_matches_recommations(&self, mut matching_data: MatchingData) -> Vec<BidOfferMatch> {
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
                    already_selected_bids.push(bid.id);
                    let selected_energy = bid.energy.min(offer.energy);
                    let new_bid_offer_match = BidOfferMatch {
                            market_id: bid.id,
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