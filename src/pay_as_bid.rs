const FLOATING_POINT_TOLERANCE: f64 = 0.00001;

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
pub struct MatchingData {
    bids: Vec<Bid>,
    offers: Vec<Offer>,
}

pub trait GetMatchesRecommendations {
    fn get_matches_recommations(&self, matching_data: MatchingData) {
        let mut bid_offer_pairs = Vec::new();
        for (market_id, data) in matching_data.iter().enumerate() {
            let mut sorted_offers = matching_data.bids.sort_by(|a, b| b.energy_rate.cmp(&a.energy_rate));
            let mut sorted_bids = matching_data.offers.sort_by(|a, b| b.energy_rate.cmp(&a.energy_rate));
    
            let mut already_selected_bids = Vec::new();
            for offer in sorted_offers:
                for bid in sorted_bids:
                    if already_selected_bids.contains(bid.id) || offer.seller == bid.buyer {
                        continue;
                    }
                    if offer.energy_rate - bid.energy_rate <= FLOATING_POINT_TOLERANCE {
                        already_selected_bids.push(bid.id);
                        let selected_energy = bid.energy.min(offer.energy);
                        let new_bid_offer_match = BidOfferMatch {
                            market_id: market_id,
                            bids: vec![bid],
                            selected_energy: selected_energy,
                            offers: vec![offer],
                            trade_rate: bid.energy_rate,
                        };
                        bid_offer_pairs.push(new_bid_offer_match);
                        break;
                    }
        }
    }
}

impl GetMatchesRecommendations for MatchingData {

}