use anyhow::{Error, Result};
use futures::StreamExt;
use subxt::{ClientBuilder, DefaultConfig, SubstrateExtrinsicParams};
use text_colorizer::*;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod gsy_node {}

pub async fn substrate_subscribe(url: String) -> Result<(), Error> {
    eprintln!("Connecting to {}", url.green());
    let api = ClientBuilder::new()
        .set_url(url.clone())
        .build()
        .await?
        .to_runtime_api::<gsy_node::RuntimeApi<DefaultConfig, SubstrateExtrinsicParams<DefaultConfig>>>();

    let mut gsy_node_events = api.events().subscribe_finalized().await?;

    while let Some(events) = gsy_node_events.next().await {
        let events = events?;
        let block_hash = events.block_hash();

        let block_number = api
            .client
            .rpc()
            .block(Some(block_hash))
            .await?
            .unwrap()
            .block
            .header
            .number;

        eprintln!("Block {:?} finalized: {:?}", block_number, block_hash);
        match (block_number as u64) % 4 == 0 {
            true =>  {
                eprintln!("Starting matching cycle")
                // TODO
            },
            _ => {}
        }
    }
    Ok(())
}
