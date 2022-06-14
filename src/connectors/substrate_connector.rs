use crate::primitives::web3::{Bid, Offer, Order, OrderSchema, OrderStatus};
use anyhow::{Error, Result};
use async_recursion::async_recursion;
use sp_keyring::AccountKeyring;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use subxt::{
    rpc::Subscription,
    sp_runtime::{generic::Header, traits::BlakeTwo256},
    ClientBuilder, DefaultConfig, PairSigner, PolkadotExtrinsicParams, SubstrateExtrinsicParams,
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

        if (block.number as u64) % 4 == 0 {
            eprintln!("{}", "Starting matching cycle".green());

            let orderbook_url_clone = Arc::clone(&orderbook_url);
            let node_url_clone = Arc::clone(&node_url);

            if let Err(error) = tokio::task::spawn(async move {
                let orderbook_url_clone = orderbook_url_clone.lock().unwrap().to_string();

                eprintln!(
                    "{} {}",
                    "Fetching orders from".green(),
                    orderbook_url_clone.clone().green().bold()
                );

                let (open_bid, open_offer) =
                    fetch_open_orders_from_orderbook_service(orderbook_url_clone)
                        .await
                        .unwrap_or_else(|e| panic!("Failed to fetch the open orders: {:?}", e));

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

            // TODO: Modify Extrinsic to SettleTrade
            tokio::task::spawn(async move {
                let node_url_clone = node_url_clone.lock().unwrap().to_string();

                let signer = PairSigner::new(AccountKeyring::Alice.pair());
                eprintln!("Signer: {:?}", signer.account_id());
                let dest = AccountKeyring::Bob.to_account_id().into();
                let api =
                    ClientBuilder::new()
                        .set_url(node_url_clone)
                        .build()
                        .await
                        .unwrap_or_else(|e| panic!("Failed to connect to node: {:?}", e))
                        .to_runtime_api::<gsy_node::RuntimeApi<
                            DefaultConfig,
                            PolkadotExtrinsicParams<DefaultConfig>,
                        >>();

                let balance_transfer = api
                    .tx()
                    .balances()
                    .transfer(dest, 10_000)
                    .unwrap_or_else(|e| panic!("Failed to create the transfer extrinsic: {:?}", e))
                    .sign_and_submit_then_watch_default(&signer)
                    .await
                    .unwrap_or_else(|e| panic!("Failed to submit the transfer extrinsic: {:?}", e))
                    .wait_for_finalized_success()
                    .await
                    .unwrap_or_else(|e| {
                        panic!(
                        "Failed to fetch a successful response for the transfer extrinsic: {:?}",
                        e
                    )
                    });

                let transfer_event = balance_transfer
                    .find_first::<gsy_node::balances::events::Transfer>()
                    .unwrap_or_else(|e| panic!(
                        "Failed to ensure that the transaction was successful by catching the associated events. {:?}",
                        e
                    ));

                eprintln!("Balance transfer success: {:?}", transfer_event);
            });
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

async fn fetch_open_orders_from_orderbook_service(
    url: String,
) -> Result<(Vec<Bid>, Vec<Offer>), Error> {
    let res = reqwest::get(url).await?;
    eprintln!("Response: {:?} {}", res.version(), res.status());
    eprintln!("Headers: {:#?}\n", res.headers());

    let body = res.json::<Vec<OrderSchema>>().await?;
    let open_bid: Vec<Bid> = body
        .clone()
        .into_iter()
        .filter(|order| {
            order.status == OrderStatus::Open && matches!(order.order, Order::Bid { .. })
        })
        .map(|order| order.into())
        .collect();
    let open_offer: Vec<Offer> = body
        .into_iter()
        .filter(|order| {
            order.status == OrderStatus::Open && matches!(order.order, Order::Offer { .. })
        })
        .map(|order| order.into())
        .collect();
    Ok((open_bid, open_offer))
}