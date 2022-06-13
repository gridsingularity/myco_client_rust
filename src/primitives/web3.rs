use codec::Encode;
use serde::{Deserialize, Serialize};
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
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, Encode, Clone, PartialEq)]
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