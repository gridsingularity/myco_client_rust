extern crate redis;
use crate::pay_as_bid::{Bid, Offer, MatchingData};
use serde_json::{Result, Value, Map};
use std::error::Error;
use chrono::{DateTime, FixedOffset};

pub fn value_to_str(value: &Value) -> String {
    match value.as_str() {
        Some(..) => value.as_str().unwrap().to_string(),
        None => String::new(),
    }
}

pub fn value_to_f32(value: &Value) -> f32 {
    match value.as_f64() {
        Some(..) => value.as_f64().unwrap() as f32,
        None => 0 as f32,
    }
}

pub fn value_to_datetime(value: &Value) -> Option<DateTime<FixedOffset>> {
    match value.as_str() {
        Some(..) => match DateTime::parse_from_rfc3339(value.as_str().unwrap()) {
            Ok(datetime) => Some(datetime),
            Err(e) => None,
        },
        None => None,
    }
}

pub fn read_bids(orders: &Value) -> Vec<Bid> {
    /// Create an array of Bid structs from the Array
    let mut bids_list = Vec::new();
    for bid in orders.as_array().unwrap() {
        let bid_struct = Bid{
            r#type: value_to_str(&bid["type"]),
            id: value_to_str(&bid["id"]),
            energy: value_to_f32(&bid["energy"]),
            energy_rate: value_to_f32(&bid["energy_rate"]),
            original_price: value_to_f32(&bid["original_price"]),
            attributes: Some(value_to_str(&bid["attributes"])),
            requirements: Some(value_to_str(&bid["requirements"])),
            buyer_origin: value_to_str(&bid["buyer_origin"]),
            buyer_origin_id: value_to_str(&bid["buyer_origin_id"]),
            buyer_id: value_to_str(&bid["buyer_id"]),
            buyer: value_to_str(&bid["buyer"]),
            time_slot: value_to_datetime(&bid["time_slot"]),
            creation_time: value_to_datetime(&bid["creation_time"]),
        };
        bids_list.push(bid_struct);
    }
    bids_list
}

pub fn read_offers(orders: &Value) -> Vec<Offer> {
    /// Create an array of Offers from the Array
    let mut offers_list = Vec::new();
    for offer in orders.as_array().unwrap() {
        let offer_struct = Offer{
            r#type: value_to_str(&offer["type"]),
            id: value_to_str(&offer["id"]),
            energy: value_to_f32(&offer["energy"]),
            energy_rate: value_to_f32(&offer["energy_rate"]),
            original_price: value_to_f32(&offer["original_price"]),
            attributes: Some(value_to_str(&offer["attributes"])),
            requirements: Some(value_to_str(&offer["requirements"])),
            seller_origin: value_to_str(&offer["seller_origin"]),
            seller_origin_id: value_to_str(&offer["seller_origin_id"]),
            seller_id: value_to_str(&offer["seller_id"]),
            seller: value_to_str(&offer["seller"]),
            time_slot: value_to_datetime(&offer["time_slot"]),
            creation_time: value_to_datetime(&offer["creation_time"]),
        };
        offers_list.push(offer_struct);
    }
    offers_list
}

pub fn process_market_id_for_pay_as_bid(market_id: &str, obj: &Value) {
    /// Create a MatchingData Struct for the pay as bid algorithm
    for (timestamp, obj) in obj.as_object().unwrap().iter() {
        let mut bids_list = Vec::new();
        let mut offers_list = Vec::new();
        for (key, orders) in obj.as_object().unwrap().iter() {
            if key == "bids" {
                bids_list = read_bids(orders);
            } else {
                offers_list = read_offers(orders);
            }
        }
        let matching_data = MatchingData{bids: bids_list, offers: offers_list};
        println!("{:?}", matching_data);
        // TODO - run the bids and offers list through the pay as bid
        // get_matches_recommendations(matching_data);
        // TODO - publish the recommendations to the appropriate channel
    }
}

pub fn unwrap_offers_bids_response(payload: &str) -> Value {
    /// When a message from the bids_offers channel is received,
    /// it extracts the market ids as keys to iterate over the
    /// corresponding sets of bids and offers and trigger the
    /// pay as bid algorithm.
    let value: Value = serde_json::from_str(&payload).unwrap();
    for (key, obj) in value.as_object().unwrap().iter() {
        if key == "bids_offers" {
            for (market_id, obj) in obj.as_object().unwrap().iter() {
                process_market_id_for_pay_as_bid(market_id, obj);
            }
        }
    };
    value
}

pub fn unwrap_recommendations_response(payload: &str) -> Value {
    /// When a message from the recommendations channel is received,
    /// it is sent to the verifier function - TODO
    let value: Value = serde_json::from_str(&payload).unwrap();
    value
}

pub fn psubscribe(channel: String) -> Result<()> //, Box<dyn Error>>
{
    let _ = tokio::spawn(async move {
        let client = redis::Client::open("redis://localhost:6379").unwrap();

        let mut con = client.get_connection().unwrap();
        let mut pubsub = con.as_pubsub();

        pubsub.psubscribe(channel).unwrap();

        loop {
            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();
            let channel_name = msg.get_channel_name();
            let unwrapped_payload = match channel_name {
                "external-myco//offers-bids/response/" => unwrap_offers_bids_response(&payload),
                "external-myco//recommendations/" => unwrap_recommendations_response(&payload),
                _ => unwrap_recommendations_response(&payload),
            };
            //println!("channel '{}': {:?}", msg.get_channel_name(), unwrapped_payload);
        }
    });

    Ok(())
}