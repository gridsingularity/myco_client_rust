use clap::Parser;
use myco_client_rust::connectors::{redis_subscribe, substrate_subscribe};
use myco_client_rust::utils::{Cli, Commands};
use text_colorizer::*;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Web2 {
            orderbook_host,
            orderbook_port
        } => async {
            let orders_response_channel = String::from("external-myco/*/offers-bids/response/");
            let recommendations_channel = String::from("external-myco/*/recommendations");
            let tick_channel = String::from("external-myco/*/events/");

            let channels = vec![
                tick_channel.clone(),
                orders_response_channel.clone(),
                recommendations_channel.clone(),
            ];

            eprintln!("Connecting to: {}:{}", orderbook_host.green(), orderbook_port.green());

            let url = format!("{}:{}", orderbook_host, orderbook_port);

            if let Err(error) = redis_subscribe(channels.clone(), url).await {
                eprintln!("{} - {:?}", "Error".red().bold(), error);
                panic!("{:?}", error);
            }
        }.await,
        Commands::Web3 {
            orderbook_host: _,
            orderbook_port: _,
            node_host,
            node_port
        } => async {
            let node_url = format!("{}:{}", node_host, node_port);
            if let Err(error) = substrate_subscribe(node_url).await {
                eprintln!("{} - {:?}", "Error".red().bold(), error);
                panic!("{:?}", error);
            }
        }.await
    }
}