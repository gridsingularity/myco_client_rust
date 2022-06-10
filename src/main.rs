use clap::Parser;
use myco_client_rust::connectors::{redis_subscribe, substrate_subscribe};
use myco_client_rust::utils::{Cli, Commands};
use std::{thread, time};
use text_colorizer::*;
use tracing::{error, info};
use myco_client_rust::utils::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let subscriber = get_subscriber("myco-client-rust".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

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

            info!("Connecting to: {}:{}", orderbook_host.green(), orderbook_port.green());

            let url = format!("{}:{}", orderbook_host, orderbook_port);

            if let Err(error) = redis_subscribe(channels.clone(), url).await {
                error!("{} - {:?}", "Error".red().bold(), error);
                panic!("{:?}", error);
            }
        }.await,
        Commands::Web3 {
            orderbook_host,
            orderbook_port,
            node_host,
            node_port
        } => async {
            let orderbook_url = format!("{}:{}/{}", orderbook_host, orderbook_port, "orders");
            let node_url = format!("{}:{}", node_host, node_port);
            if let Err(error) = substrate_subscribe(orderbook_url.clone(), node_url.clone()).await {
                info!("{} - {:?}", "Error".bright_red().bold(), error);
                let mut attempt: u8 = 1;
                while attempt <= cli.max_attempts {
                    info!("{}\n{}: {}", "Retrying...".yellow(), "Attempt".yellow(), attempt.to_string().bright_white().bold());
                    let two_seconds = time::Duration::from_millis(2000);
                    thread::sleep(two_seconds);
                    if let Err(error) = substrate_subscribe(orderbook_url.clone(), node_url.clone()).await {
                        error!("{} - {:?}", "Error".bright_red().bold(), error);
                        attempt += 1;
                    }
                }
            }
        }.await
    }
}