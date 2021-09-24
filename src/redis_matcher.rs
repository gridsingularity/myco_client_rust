extern crate redis;
use std::thread;

pub struct RedisConnector {
    simulation_id: String,
    pubsub_thread: String,
    redis_db: String,
}

pub trait RedisBaseMatcher {
    fn request_offers_bids(&self);
    fn on_offers_bids_response(&self);
    fn on_matched_recommendations_response(&self);
    fn submit_matches(&self);
    fn on_tick(&self);
    fn on_market_cycle(&self);
    fn on_finish(&self);
    fn on_event_or_response(&self);
}

impl RedisBaseMatcher for RedisConnector {

}

fn main() {
    let client = redis::Client::open("redis://localhost:6379")?;
    let mut con = client.get_connection()?;
}