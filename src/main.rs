extern crate redis;
mod redis_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = String::from("external-myco/*/recommendations/");

    if let Err(error) = redis_subscriber::psubscribe(channel.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to channel: {}", channel);
    }

    Ok(())
}