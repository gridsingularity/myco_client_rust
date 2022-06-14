use crate::algorithms::PayAsBid;
use crate::primitives::web3::{
    Bid, BidOfferMatch, MatchingData, Offer, Order, OrderSchema, OrderStatus,
};
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
use tracing::{error, info};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod gsy_node {}

#[async_recursion]
pub async fn substrate_subscribe(orderbook_url: String, node_url: String) -> Result<(), Error> {
    info!("Connecting to {}", node_url);

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
            info!("Starting matching cycle");

            let orderbook_url_clone = Arc::clone(&orderbook_url);
            let node_url_clone = Arc::clone(&node_url);

            let matches_clone_one = Arc::clone(&matches);
            let matches_clone_two = Arc::clone(&matches);

            if let Err(error) = tokio::task::spawn(async move {
                let orderbook_url_clone = orderbook_url_clone.lock().unwrap().to_string();

                info!(
                    "Fetching orders from {}",
                    orderbook_url_clone.clone()
                );

                let (open_bid, open_offer) =
                    fetch_open_orders_from_orderbook_service(orderbook_url_clone)
                        .await
                        .unwrap_or_else(|e| panic!("Failed to fetch the open orders: {:?}", e));

                info!("Open Bid - {:?}", open_bid);
                info!("Open Offer - {:?}", open_offer);

                let mut matching_data = MatchingData {
                    bids: open_bid,
                    offers: open_offer,
                    market_id: 1,
                };
                let bid_offer_matches = matching_data.pay_as_bid();
                matches_clone_one.lock().unwrap().extend(bid_offer_matches);
                info!(
                    "Matches - {:?}",
                    matches_clone_one.lock().unwrap()
                );
            })
            .await
            {
                error!(
                    "Error while fetching the orderbook - {:?}",
                    error
                );
            }

            tokio::task::spawn(async move {
                info!(
                    "Settling following matches - {:?}",
                    matches_clone_two.lock().unwrap()
                );

                let node_url_clone = node_url_clone.lock().unwrap().to_string();
                let matches: Vec<BidOfferMatch> = matches_clone_two.lock().unwrap().clone();

                if let Ok(()) = send_settle_trades_extrinsic(
                    node_url_clone,
                    matches,
                ).await {
                    info!("Settling trades successful");
                } else {
                    error!("Settling trades failed");
                }
            });
        }
    }
    error!("Subscription dropped.");
    loop {
        info!("Trying to reconnect...");
        let two_seconds = time::Duration::from_millis(2000);
        thread::sleep(two_seconds);
        let orderbook_url = orderbook_url.lock().unwrap().to_string();
        let node_url = node_url.lock().unwrap().to_string();
        if let Err(error) = substrate_subscribe(orderbook_url, node_url.clone()).await {
            error!("Error - {:?}", error);
        }
    }
}

async fn fetch_open_orders_from_orderbook_service(
    url: String,
) -> Result<(Vec<Bid>, Vec<Offer>), Error> {
    let res = reqwest::get(url).await?;
    info!("Response: {:?} {}", res.version(), res.status());
    info!("Headers: {:#?}\n", res.headers());

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

async fn send_settle_trades_extrinsic(
    url: String,
    _matches: Vec<BidOfferMatch>,
) -> Result<(), Error> {
    let signer = PairSigner::new(AccountKeyring::Alice.pair());
    info!("Signer: {:?}", signer.account_id());
    let dest = AccountKeyring::Bob.to_account_id().into();
    let api = ClientBuilder::new()
        .set_url(url)
        .build()
        .await?
        .to_runtime_api::<gsy_node::RuntimeApi<
            DefaultConfig,
            PolkadotExtrinsicParams<DefaultConfig>,
        >>();

    // TODO: Modify Extrinsic to SettleTrade
    let balance_transfer = api
        .tx()
        .balances()
        .transfer(dest, 10_000)?
        .sign_and_submit_then_watch_default(&signer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let transfer_event = balance_transfer
        .find_first::<gsy_node::balances::events::Transfer>()?;

    if let Some(event) = transfer_event {
        info!("Trade successfully cleared: {:?}", event);
    } else {
        error!("Failed to clear the trade");
    }

    Ok(())
}