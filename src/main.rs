use myco_client_rust::connectors::psubscribe;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders_response_channel = String::from("external-myco/*/offers-bids/response/");
    let recommendations_channel = String::from("external-myco/*/recommendations");
    let tick_channel = String::from("external-myco/*/events/");

    let channels = vec![
        tick_channel.clone(),
        orders_response_channel.clone(),
        recommendations_channel.clone()
    ];

    if let Err(error) = psubscribe(channels.clone()) {
        println!("{:?}", error);
        panic!("{:?}", error);
    } else {
        println!("subscribed to the following channels:");
        for channel in channels {
            println!("{}", channel);
        }
    }

    Ok(())
}