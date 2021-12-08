extern crate redis;
use redis::{Client, Commands, Connection, RedisResult};
use std::thread;
mod redis_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://localhost:6379").unwrap();
    let mut conn = client.get_connection().unwrap();

    if let Err(error) = redis_subscriber::subscribe(String::from("external-myco/*/offers-bids/response/")) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("connected to queue");
    }

    Ok(())
}