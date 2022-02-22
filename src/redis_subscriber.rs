extern crate redis;

use crate::pay_as_bid::{Bid, Offer, MatchingData, GetMatchesRecommendations};
use std::env;

use serde_json::{Result, Value};
use chrono::{NaiveDateTime};
use crate::redis::Commands;

pub fn value_to_str(value: &Value) -> String {
    // Helper function to convert the serde Value to String
    match value.as_str() {
        Some(..) => value.as_str().unwrap().to_string(),
        None => String::new(),
    }
}

pub fn value_to_f32(value: &Value) -> f32 {
    // Helper function to convert the serde Value to f32
    match value.as_f64() {
        Some(..) => value.as_f64().unwrap() as f32,
        None => 0 as f32,
    }
}

pub fn value_to_datetime(value: &Value) -> Option<NaiveDateTime> {
    // Helper function to convert the serde Value to NaiveDateTime
    match value.as_str() {
        Some(..) => match NaiveDateTime::parse_from_str(value.as_str().unwrap(), "%Y-%m-%dT%H:%M:%S") {
            Ok(datetime) => Some(datetime),
            Err(_e) => None,
        },
        None => None,
    }
}

pub fn read_bids(orders: &Value) -> Vec<Bid> {
    // Create an array of Bid structs from the serde Value
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
    // Create an array of Offers from the serde Value
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

pub fn process_market_id_for_pay_as_bid(obj: &Value) {
    // Create a MatchingData Struct for the pay as bid algorithm
    for (_timestamp, obj) in obj.as_object().unwrap().iter() {
        let mut bids_list = Vec::new();
        let mut offers_list = Vec::new();
        let mut market_id = String::new();
        for (key, orders) in obj.as_object().unwrap().iter() {
            if key == "bids" {
                bids_list = read_bids(orders);
            } else if key == "offers" {
                offers_list = read_offers(orders);
            } else if key == "market_id" {
                market_id = orders.to_string();
            } else {
                panic!("Unable to process market id: key not in ['bids', 'offers', 'market_id'].")
            }
        }
        let mut matching_data = MatchingData{
            bids: bids_list, offers: offers_list, market_id
        };
        // TODO - run the bids and offers list through the pay as bid
        let algorithm_result = matching_data.get_matches_recommendations();
        println!("ALGORITHM RESULT: {:?}", algorithm_result);
        // TODO - add tests for the result 
        // TODO - publish the recommendations to the appropriate channel
    }
}

pub fn unwrap_offers_bids_response(payload: &str) -> Value {
    // When a message from the bids_offers channel is received,
    // it extracts the market ids as keys to iterate over the
    // corresponding sets of bids and offers and trigger the
    // pay as bid algorithm.
    let value: Value = serde_json::from_str(&payload).unwrap();
    for (key, obj) in value.as_object().unwrap().iter() {
        if key == "bids_offers" {
            for (_market_id, obj) in obj.as_object().unwrap().iter() {
                process_market_id_for_pay_as_bid(obj);
            }
        }
    };
    value
}

pub fn unwrap_recommendations_response(payload: &str) -> Value {
    // When a message from the recommendations channel is received,
    // it is sent to the verifier function - TODO
    // Will be sent to TradeSettlement pallet, rejected matches are sent back by the OCW - TODO
    let value: Value = serde_json::from_str(&payload).unwrap();
    value
}

pub fn unwrap_tick_response(payload: &str, client: &redis::Client) -> Value {
    // When a message from the tick channel is received,
    // we check the slot completion %
    let value: Value = serde_json::from_str(&payload).unwrap();
    for (key, obj) in value.as_object().unwrap().iter() {
        if key == "slot_completion" {
            let slot_percent_str: &str = &obj.as_str().unwrap();
            let length = slot_percent_str.len();
            let slot_percent_int: i32 = slot_percent_str[..length-1].parse().unwrap();
            if (slot_percent_int > 33 && slot_percent_int < 66) || (slot_percent_int >= 66) {
                client.get_connection().unwrap().publish::<String, String, redis::Value>(
                    "external-myco//offers-bids/".to_string(), "{}".to_string());
            }
        }
    }
    value
}

pub fn psubscribe(channels: Vec<String>) -> Result<()>
{
    let _ = tokio::spawn(async move {
        let redis_url = env::var("REDIS_URL").unwrap_or("localhost".to_string());

        let client = redis::Client::open(
            format!("redis://{}:6379", redis_url)).unwrap();

        let mut con = client.get_connection().unwrap();

        let mut pubsub = con.as_pubsub();
        for channel in channels {
            pubsub.psubscribe(channel).unwrap();
        }

        loop {
            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();
            let channel_name = msg.get_channel_name();
            let _unwrapped_payload = match channel_name {
                "external-myco//offers-bids/response/" => unwrap_offers_bids_response(&payload),
                "external-myco//recommendations/" => unwrap_recommendations_response(&payload),
                "external-myco//events/" => unwrap_tick_response(&payload, &client),
                _ => unwrap_recommendations_response(&payload),
            };
        }
    });

    Ok(())
}