extern crate redis;

use std::error::Error;

pub fn subscribe(channel: String) -> Result<(), Box<dyn Error>> {
    let _ = tokio::spawn(async move {
        let client = redis::Client::open("redis://localhost:6379").unwrap();

        let mut con = client.get_connection().unwrap();
        let mut pubsub = con.as_pubsub();

        pubsub.psubscribe(channel).unwrap();

        loop {
            let msg = pubsub.get_message().unwrap();
            let payload : String = msg.get_payload().unwrap();
            println!("channel '{}': {}", msg.get_channel_name(), payload);
        }
    });

    Ok(())
}