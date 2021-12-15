extern crate redis;
mod redis_subscriber;
mod pay_as_bid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel1 = String::from("external-myco/*/offers-bids/response/");
    let channel2 = String::from("external-myco/*/recommendations");

    if let Err(error) = redis_subscriber::psubscribe(channel1.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to channel: {}", channel1);
    }

    if let Err(error) = redis_subscriber::psubscribe(channel2.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to channel: {}", channel2);
    }

    Ok(())
}