extern crate redis;
mod redis_subscriber;
mod pay_as_bid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders_response_channel = String::from("external-myco/*/offers-bids/response/");
    let recommendations_channel = String::from("external-myco/*/recommendations");
    let tick_channel = String::from("external-myco/*/events/");

    if let Err(error) = redis_subscriber::psubscribe(orders_response_channel.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to channel: {}", orders_response_channel);
    }

    if let Err(error) = redis_subscriber::psubscribe(recommendations_channel.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to channel: {}", recommendations_channel);
    }

    if let Err(error) = redis_subscriber::psubscribe(tick_channel.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to channel: {}", tick_channel);
    }

    Ok(())
}