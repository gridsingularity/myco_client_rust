extern crate redis;
use crate::pay_as_bid::{Bid, Offer, MatchingData};
use serde_json::{Result, Value, Map};
use std::error::Error;

pub fn read_bids(bids_offers: &Value) -> Vec<Bid> {
    /// Create an array of Bid structs from the Array
    let mut bids_list = Vec::new();
    bids_list
}

pub fn read_offers(bids_offers: &Value) -> Vec<Offer> {
    /// Create an array of Offers from the Array
    let mut offers_list = Vec::new();
    offers_list
}

pub fn process_market_id_for_pay_as_bid(market_id: &str, obj: &Value) {
    /// Create a MatchingData Struct for the pay as bid algorithm
    println!("Market ID {} and Object {:?}", market_id, obj);
    for (timestamp, obj) in obj.as_object().unwrap().iter() {
        let mut bids_list = Vec::new();
        let mut offers_list = Vec::new();
        for (key, bids_offers) in obj.as_object().unwrap().iter() {
            if key == "bids" {
                bids_list = read_bids(bids_offers);
            } else {
                offers_list = read_offers(bids_offers);
            }
        }
        let matching_data = MatchingData{bids: bids_list, offers: offers_list};
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
            println!("Market ID: {}", key); // TODO - remove this line
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