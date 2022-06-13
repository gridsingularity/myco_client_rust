use crate::primitives::web3::{Bid, Offer, Order, OrderSchema, OrderStatus};
use anyhow::{Error, Result};
use async_recursion::async_recursion;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use subxt::{
    rpc::Subscription,
    sp_runtime::{generic::Header, traits::BlakeTwo256},
    ClientBuilder, DefaultConfig, SubstrateExtrinsicParams,
};
use text_colorizer::*;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod gsy_node {}

#[async_recursion]
pub async fn substrate_subscribe(orderbook_url: String, node_url: String) -> Result<(), Error> {
    eprintln!("{} {}", "Connecting to".green(), node_url.green().bold());

    let api = ClientBuilder::new()
        .set_url(node_url.clone())
        .build()
        .await?
        .to_runtime_api::<gsy_node::RuntimeApi<DefaultConfig, SubstrateExtrinsicParams<DefaultConfig>>>();

    let mut gsy_blocks_events: Subscription<Header<u32, BlakeTwo256>> =
        api.client.rpc().subscribe_finalized_blocks().await?;

    let orderbook_url = Arc::new(Mutex::new(orderbook_url));
    let node_url = Arc::new(Mutex::new(node_url.clone()));

    while let Some(Ok(block)) = gsy_blocks_events.next().await {
        eprintln!("Block {:?} finalized: {:?}", block.number, block.hash());

        if (block.number as u64) % 1 == 0 {
            eprintln!("{}", "Starting matching cycle".green());

            let orderbook_url_clone = Arc::clone(&orderbook_url);

            if let Err(error) = tokio::task::spawn(async move {
                let orderbook_url_clone = orderbook_url_clone.lock().unwrap().to_string();

                eprintln!(
                    "{} {}",
                    "Fetching orders from".green(),
                    orderbook_url_clone.clone().green().bold()
                );

                let res = reqwest::get(orderbook_url_clone)
                    .await
                    .expect("Failed to get orderbook");
                eprintln!("Response: {:?} {}", res.version(), res.status());
                eprintln!("Headers: {:#?}\n", res.headers());

                let body = res
                    .json::<Vec<OrderSchema>>()
                    .await
                    .expect("Failed to parse response");
                let open_bid: Vec<Bid> = body
                    .clone()
                    .into_iter()
                    .filter(|order| order.status == OrderStatus::Open && matches!(order.order, Order::Bid{..}))
                    .map(|order| order.into()).collect();
                let open_offer: Vec<Offer> = body
                    .into_iter()
                    .filter(|order| order.status == OrderStatus::Open && matches!(order.order, Order::Offer{..}))
                    .map(|order| order.into()).collect();
                eprintln!("{} - {:?}", "Open Bid".blue(), open_bid);
                eprintln!("{} - {:?}", "Open Offer".magenta(), open_offer);
            })
            .await
            {
                eprintln!(
                    "{} - {:?}",
                    "Error while fetching the orderbook".red(),
                    error
                );
            }

            // TODO: Add Extrinsic creation and broadcast
        }
    }
    eprintln!("{}", "Subscription dropped.".bright_red().bold());
    loop {
        eprintln!("{}", "Trying to reconnect...".yellow());
        let two_seconds = time::Duration::from_millis(2000);
        thread::sleep(two_seconds);
        let orderbook_url = orderbook_url.lock().unwrap().to_string();
        let node_url = node_url.lock().unwrap().to_string();
        if let Err(error) = substrate_subscribe(orderbook_url, node_url.clone()).await {
            eprintln!("{} - {:?}", "Error".bright_red().bold(), error);
        }
    }
}
