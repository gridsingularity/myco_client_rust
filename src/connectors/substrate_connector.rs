use crate::primitives::web3::{Bid, MatchingData, Offer, Order, OrderSchema, OrderStatus};
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
use tracing::{error, info};
use crate::algorithms::PayAsBid;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod gsy_node {}

#[async_recursion]
pub async fn substrate_subscribe(orderbook_url: String, node_url: String) -> Result<(), Error> {
    info!("{} {}", "Connecting to".green(), node_url.green().bold());

    let api = ClientBuilder::new()
        .set_url(node_url.clone())
        .build()
        .await?
        .to_runtime_api::<gsy_node::RuntimeApi<DefaultConfig, SubstrateExtrinsicParams<DefaultConfig>>>();

    let mut gsy_blocks_events: Subscription<Header<u32, BlakeTwo256>> =
        api.client.rpc().subscribe_finalized_blocks().await?;

    let orderbook_url = Arc::new(Mutex::new(orderbook_url));
    let node_url = Arc::new(Mutex::new(node_url.clone()));

    let matches = Arc::new(Mutex::new(Vec::new()));

    while let Some(Ok(block)) = gsy_blocks_events.next().await {
        info!("Block {:?} finalized: {:?}", block.number, block.hash());

        if (block.number as u64) % 4 == 0 {
            info!("{}", "Starting matching cycle".green());

            let orderbook_url_clone = Arc::clone(&orderbook_url);
            let node_url_clone = Arc::clone(&node_url);

            let matches_clone_one = Arc::clone(&matches);
            let matches_clone_two = Arc::clone(&matches);

            if let Err(error) = tokio::task::spawn(async move {
                let orderbook_url_clone = orderbook_url_clone.lock().unwrap().to_string();

                info!(
                    "{} {}",
                    "Fetching orders from".green(),
                    orderbook_url_clone.clone().green().bold()
                );

                let res = reqwest::get(orderbook_url_clone)
                    .await
                    .expect("Failed to get orderbook");
                info!("Response: {:?} {}", res.version(), res.status());
                info!("Headers: {:#?}\n", res.headers());

                let body = res
                    .json::<Vec<OrderSchema>>()
                    .await
                    .expect("Failed to parse response");
                let open_bid: Vec<Bid> = body
                    .clone()
                    .into_iter()
                    .filter(|order| {
                        order.status == OrderStatus::Open
                            && matches!(order.order, Order::Bid { .. })
                    })
                    .map(|order| order.into())
                    .collect();
                let open_offer: Vec<Offer> = body
                    .into_iter()
                    .filter(|order| {
                        order.status == OrderStatus::Open
                            && matches!(order.order, Order::Offer { .. })
                    })
                    .map(|order| order.into())
                    .collect();
                info!("{} - {:?}", "Open Bid".blue(), open_bid);
                info!("{} - {:?}", "Open Offer".magenta(), open_offer);
                let mut matching_data = MatchingData {
                    bids: open_bid,
                    offers: open_offer,
                    market_id: 1,
                };
                let bid_offer_matches = matching_data.pay_as_bid();
                matches_clone_one.lock().unwrap().extend(bid_offer_matches);
                info!("{} - {:?}", "Matches".green(), matches_clone_one.lock().unwrap());
            })
            .await
            {
                error!(
                    "{} - {:?}",
                    "Error while fetching the orderbook".red(),
                    error
                );
            }

            // TODO: Modify Extrinsic to SettleTrade
            tokio::task::spawn(async move {
                info!("{} - {:?}", "Settling following matches".green(), matches_clone_two.lock().unwrap());
                let node_url_clone = node_url_clone.lock().unwrap().to_string();

                let signer = PairSigner::new(AccountKeyring::Alice.pair());
                info!("Signer: {:?}", signer.account_id());
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

                info!("Balance transfer success: {:?}", transfer_event);
            });
        }
    }
    error!("{}", "Subscription dropped.".bright_red().bold());
    loop {
        info!("{}", "Trying to reconnect...".yellow());
        let two_seconds = time::Duration::from_millis(2000);
        thread::sleep(two_seconds);
        let orderbook_url = orderbook_url.lock().unwrap().to_string();
        let node_url = node_url.lock().unwrap().to_string();
        if let Err(error) = substrate_subscribe(orderbook_url, node_url.clone()).await {
            error!("{} - {:?}", "Error".bright_red().bold(), error);
        }
    }
}
