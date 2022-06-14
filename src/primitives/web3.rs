use crate::algorithms::PayAsBid;
use codec::Encode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use subxt::sp_core::H256;
use subxt::sp_runtime::traits::{BlakeTwo256, Hash};

#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum Order {
    Bid(Bid),
    Offer(Offer),
}

impl Order {
    pub fn hash(&self) -> H256 {
        BlakeTwo256::hash_of(self)
    }
}
/// Order component struct
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq, PartialOrd)]
pub struct OrderComponent {
    pub energy: u32,
    pub energy_rate: u32,
    pub pref_partners: Option<Vec<String>>,
    pub priority: u32,
    pub energy_type: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq)]
pub struct OrderSchema {
    pub _id: H256,
    pub status: OrderStatus,
    pub order: Order,
}

impl From<Order> for OrderSchema {
    fn from(order: Order) -> Self {
        OrderSchema {
            _id: order.hash(),
            status: Default::default(),
            order,
        }
    }
}

impl From<OrderSchema> for Order {
    fn from(order: OrderSchema) -> Self {
        order.order
    }
}

/// Order status
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq)]
pub enum OrderStatus {
    Open,
    Executed,
    Deleted,
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Open
    }
}

/// Bid order struct
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq, PartialOrd)]
pub struct Bid {
    pub buyer: String,
    pub uuid: u8,
    pub market_uuid: Option<Vec<u8>>,
    pub time_slot: u64,
    pub creation_time: Option<u64>,
    pub attributes: Vec<Vec<u8>>,
    pub bid_component: OrderComponent,
}

impl From<Order> for Bid {
    fn from(order: Order) -> Self {
        match order {
            Order::Bid(bid) => bid,
            _ => panic!("Expected Order::Bid"),
        }
    }
}

impl From<OrderSchema> for Bid {
    fn from(order: OrderSchema) -> Self {
        match order.order {
            Order::Bid(bid) => bid,
            _ => panic!("Expected Order::Bid"),
        }
    }
}

/// Offer (Ask) order struct
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq, PartialOrd)]
pub struct Offer {
    pub seller: String,
    pub uuid: u8,
    pub market_uuid: Option<Vec<u8>>,
    pub time_slot: u64,
    pub creation_time: Option<u64>,
    pub attributes: Vec<Vec<u8>>,
    pub offer_component: OrderComponent,
}

impl From<Order> for Offer {
    fn from(order: Order) -> Self {
        match order {
            Order::Offer(offer) => offer,
            _ => panic!("Expected Order::Offer"),
        }
    }
}

impl From<OrderSchema> for Offer {
    fn from(order: OrderSchema) -> Self {
        match order.order {
            Order::Offer(offer) => offer,
            _ => panic!("Expected Order::Offer"),
        }
    }
}

#[derive(Debug, Encode, Clone, PartialEq)]
pub struct BidOfferMatch {
    pub market_id: u8,
    pub time_slot: u64,
    pub bid: Bid,
    pub offer: Offer,
    pub residual_offer: Option<Offer>,
    pub residual_bid: Option<Bid>,
    pub selected_energy: u32,
    pub energy_rate: u32,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MatchingData {
    pub bids: Vec<Bid>,
    pub offers: Vec<Offer>,
    pub market_id: u8,
}

impl PayAsBid for MatchingData {
    type Output = BidOfferMatch;

    fn pay_as_bid(&mut self) -> Vec<Self::Output> {
        let mut bid_offer_pairs = Vec::new();

        self.bids
            .sort_by(|a, b| b.bid_component.energy_rate.partial_cmp(&a.bid_component.energy_rate).unwrap());
        self.offers
            .sort_by(|a, b| b.offer_component.energy_rate.partial_cmp(&a.offer_component.energy_rate).unwrap());

        let mut available_order_energy: HashMap<u8, u32> = HashMap::new();
        for offer in self.offers.clone() {
            for bid in self.bids.clone() {
                if offer.seller == bid.buyer {
                    continue;
                }

                if offer.offer_component.energy_rate - bid.bid_component.energy_rate
                    > 0
                {
                    continue;
                }

                if !available_order_energy.contains_key(&bid.uuid) {
                    available_order_energy
                        .insert(bid.uuid.clone(), bid.bid_component.energy)
                        .unwrap_or(0);
                }
                if !available_order_energy.contains_key(&offer.uuid) {
                    available_order_energy
                        .insert(offer.uuid.clone(), offer.offer_component.energy)
                        .unwrap_or(0);
                }

                let offer_energy = available_order_energy.get(&offer.uuid).unwrap().clone();
                let bid_energy = available_order_energy.get(&bid.uuid).unwrap().clone();

                let selected_energy = offer_energy.min(bid_energy);

                if selected_energy <= 0 {
                    continue;
                }

                available_order_energy.insert(bid.uuid.clone(), bid_energy - selected_energy);
                available_order_energy.insert(offer.uuid.clone(), offer_energy - selected_energy);

                let new_bid_offer_match = BidOfferMatch {
                    market_id: self.market_id.clone(),
                    time_slot: offer.time_slot,
                    bid: bid.clone(),
                    selected_energy,
                    offer: offer.clone(),
                    residual_offer: None,
                    residual_bid: None,
                    energy_rate: bid.bid_component.energy_rate,
                };
                bid_offer_pairs.push(new_bid_offer_match);

                if let Some(offer_residual_energy) = available_order_energy.get(&offer.uuid) {
                    if *offer_residual_energy <= 0 {
                        break;
                    }
                }
            }
        }
        bid_offer_pairs
    }
}
