use log::info;
use rand::prelude::IndexedRandom;
use tokio::time::{sleep, Duration};
use uqoin_core::state::BlockInfo;
use uqoin_core::blockchain::Blockchain;

use crate::utils::*;
use crate::scopes::blockchain::BlockData;


const SYNC_TIMEOUT: u64 = 5000;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    // Random generator
    let mut rng = rand::rng();

    loop {
        // Sync timeout
        sleep(Duration::from_millis(SYNC_TIMEOUT)).await;

        // Choose a random node
        if let Some(random_node) = appdata.nodes.read().await.choose(&mut rng) {
            // Check remote blocks
            // TODO: Check remote block before syncing...

            info!("Try to sync with {}", random_node);

            // Request last block info of the node
            let last_info_remote = get_remote_block_info(
                &random_node, None
            ).await?;

            // Get local last block info
            let last_info_local: BlockInfo = appdata.state.read().await
                                                .get_last_block_info().clone();

            // Synchronize basic condition (remote transaction count is greater
            // than the local one)
            if last_info_remote.offset > last_info_local.offset {
                info!("Syncing with {}", random_node);

                // Calculate `bix` where the chains diverged
                let bix_sync = calculate_divergence(
                    last_info_local.bix, last_info_remote.bix, &random_node,
                    &*appdata.blockchain.read().await
                ).await?;

                // Blocking `blockchain`, `state` and `pool` objects
                let blockchain = appdata.blockchain.write().await;
                let mut state = appdata.state.write().await;
                let mut pool = appdata.pool.write().await;

                // Rolling back local blockchain until `bix_sync`
                for bix in ((bix_sync + 1) ..= last_info_local.bix).rev() {
                    info!("Rolling back the block with bix = {}", bix);
                    let block = blockchain.get_block(bix).await?;
                    let transactions = blockchain
                        .get_transactions_of_block(&block).await?;
                    state.roll_down(bix, &block, &transactions, 
                                    &appdata.schema);
                    pool.roll_down(&transactions, &appdata.schema, &state);
                }
                blockchain.truncate(bix_sync).await?;

                // Rolling up local blockchain until `roll_up_bix` requesting
                // the remote node
                for bix in (bix_sync + 1) ..= last_info_remote.bix {
                    info!("Rolling up the block with bix = {}", bix);
                    let block_data = get_remote_block_data(&random_node, 
                                                           Some(bix)).await?;
                    blockchain.push_new_block(&block_data.block, 
                                              &block_data.transactions).await?;
                    state.roll_up(bix, &block_data.block, 
                                  &block_data.transactions, &appdata.schema);
                    pool.roll_up(&block_data.transactions, &appdata.schema, 
                                 &state);
                }

                info!("Synced with {} successfully", random_node);
            }
        }
    }
}


async fn calculate_divergence(bix_last_local: u64, bix_last_remote: u64,  
                              node: &str, blockchain: &Blockchain) -> 
                              TokioResult<u64> {
    // Minimum `bix`
    let mut bix = std::cmp::min(bix_last_local, bix_last_remote);

    // Loop backward to compare hashes
    while bix > 0 {
        // Get remove blockchain hash as `bix`
        let hash_remote = get_remote_block_info(
            node, Some(bix)
        ).await?.hash;

        // Get local blockchain hash as `bix`
        let hash_local = blockchain.get_block(bix).await?.hash;

        // If hashes is equal, leave the loop
        if hash_local == hash_remote {
            break;
        }

        // Decrement `bix` switching to the previous block
        bix -= 1;
    }

    // Return maximum `bix` until which the chains are equal
    Ok(bix)
}


async fn get_remote_block_info(node: &str, bix: Option<u64>) -> 
                               TokioResult<BlockInfo> {
    let url = if let Some(bix) = bix {
        format!("{}/blockchain/block-info?bix={}", node, bix)
    } else {
        format!("{}/blockchain/block-info", node)
    };
    let resp = reqwest::get(url).await.unwrap();
    let content = resp.text().await.unwrap();
    let block_info = serde_json::from_str(&content)?;
    Ok(block_info)
}


async fn get_remote_block_data(node: &str, bix: Option<u64>) -> 
                               TokioResult<BlockData> {
    let url = if let Some(bix) = bix {
        format!("{}/blockchain/block-data?bix={}", node, bix)
    } else {
        format!("{}/blockchain/block-data", node)
    };
    let resp = reqwest::get(url).await.unwrap();
    let content = resp.text().await.unwrap();
    let block_info = serde_json::from_str(&content)?;
    Ok(block_info)
}
