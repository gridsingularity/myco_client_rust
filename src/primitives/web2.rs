use serde::{Serialize, Deserialize, Serializer};
use chrono::{NaiveDateTime};

pub fn serialize_datetime<S>(
    datetime: &Option<NaiveDateTime>,
    serializer: S
) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
    const FORMAT: &'static str = "%Y-%m-%dT%H:%M";
    match datetime {
        Some(datetime) => {
            let s = format!("{}", datetime.format(FORMAT));
            serializer.serialize_str(&s)
        },
        None => serializer.serialize_none()
    }
}

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
    #[serde(serialize_with = "serialize_datetime")]
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
    #[serde(serialize_with = "serialize_datetime")]
    pub time_slot: Option<NaiveDateTime>,
    pub creation_time: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BidOfferMatch {
    pub market_id: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub time_slot: Option<NaiveDateTime>,
    pub bid: Bid,
    pub selected_energy: f32,
    pub offer: Offer,
    pub trade_rate: f32,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MatchingData {
    pub bids: Vec<Bid>,
    pub offers: Vec<Offer>,
    pub market_id: String
}