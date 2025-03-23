use log::info;
use rand::prelude::IndexedRandom;
use tokio::time::{sleep, Duration};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uqoin_core::block::{BlockInfo, BlockData};
use uqoin_core::blockchain::Blockchain;
use uqoin_core::state::State;
use uqoin_core::pool::Pool;

use crate::utils::*;
use crate::scopes::blockchain::BlockQuery;
use crate::tasks::mine::COMPLEXITY;


const SYNC_TIMEOUT: u64 = 5000;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    // Random generator
    let mut rng = rand::rng();

    loop {
        // Sync timeout
        sleep(Duration::from_millis(SYNC_TIMEOUT)).await;

        // Choose a random node
        if let Some(random_node) = appdata.nodes.read().await.choose(&mut rng) {
            info!("Trying to sync with {}", random_node);

            // Request last block info of the node
            let last_info_remote: BlockInfo = request_node(
                &random_node, "/blockchain/block-info", None::<BlockQuery>
            ).await?;

            // Get local last block info
            let last_info_local: BlockInfo = appdata.state.read().await
                                                .get_last_block_info().clone();

            // Synchronize basic condition (remote transaction count is greater
            // than the local one)
            if last_info_remote.offset > last_info_local.offset {
                info!("Need to sync with {}", random_node);

                // Request for divergent blocks
                let blocks = request_for_divergent_blocks(
                    last_info_local.bix, last_info_remote.bix, &random_node,
                    &*appdata.blockchain.read().await
                ).await?;

                // Check divergent blocks
                if let Some((state_new, pool_new)) = 
                        check_divergent_blocks(&blocks, &appdata).await? {
                    info!("Syncing with {}", random_node);

                    // Lock blockchain, state and pool
                    let blockchain = appdata.blockchain.write().await;
                    let mut state = appdata.state.write().await;
                    let mut pool = appdata.pool.write().await;

                    // Migrate blockchain
                    migrate_blockchain(&blocks, &blockchain).await?;

                    // Update state
                    *state = state_new;

                    // Merge pool
                    pool.merge(&pool_new, &appdata.schema, &state);

                    info!("Synced with {} successfully", random_node);
                }
            }
        }
    }
}


async fn request_node<T: DeserializeOwned, Q: Serialize>(
        node: &str, path: &str, qs: Option<Q>) -> TokioResult<T> {
    let query = qs.map(|q| serde_qs::to_string(&q).unwrap());
    let url = if let Some(query) = query {
        format!("{}{}?{}", node, path, query)
    } else {
        format!("{}{}", node, path)
    };
    let resp = reqwest::get(url).await.unwrap();
    let content: String = resp.text().await.unwrap();
    let instance = serde_json::from_str::<T>(&content)?;
    Ok(instance)
}


async fn request_for_divergent_blocks(bix_last_local: u64, bix_last_remote: u64, 
                                      node: &str, blockchain: &Blockchain) -> 
                                      TokioResult<Vec<BlockData>> {
    // Block number (`bix`) to download
    let mut bix = bix_last_remote;

    // Divergent block vector
    let mut blocks: Vec<BlockData> = Vec::new();

    // Download forward blocks
    while bix > bix_last_local {
        // Get remote block data
        let block_data: BlockData = request_node(
            node, "/blockchain/block-data",
            Some(BlockQuery { bix: Some(bix) })
        ).await?;

        // Push the block data
        blocks.push(block_data);

        // Decrement `bix`
        bix -= 1;
    }

    // Download divergent blocks
    while bix > 0 {
        // Get remote block data
        let block_data: BlockData = request_node(
            node, "/blockchain/block-data",
            Some(BlockQuery { bix: Some(bix) })
        ).await?;

        // Get local block hash for the `bix`
        let hash_local = blockchain.get_block(bix).await?.hash;

        // Continue if hashes are different else break the loop
        if block_data.block.hash != hash_local {
            // Push the block data
            blocks.push(block_data);

            // Decrement `bix` and continue
            bix -= 1;
        } else {
            break;
        }
    }

    // Return divergent blocks in reversed (historical) order
    Ok(blocks.into_iter().rev().collect())
}


async fn check_divergent_blocks(blocks: &[BlockData], appdata: &WebAppData) -> 
                                TokioResult<Option<(State, Pool)>> {
    // Get blockchain and clone the current state and pool
    let blockchain = appdata.blockchain.read().await;
    let mut state = appdata.state.read().await.clone();
    let mut pool = appdata.pool.read().await.clone();

    let bix_sync = blocks[0].bix - 1;

    let mut bix = blockchain.get_block_count().await?;

    // Roll down the state and pool with local blocks
    while bix > bix_sync {
        // Get local block data
        let block_data = blockchain.get_block_data(bix).await?;

        // Roll back state and pool
        state.roll_down(bix, &block_data.block, &block_data.transactions, 
                        &appdata.schema);
        pool.roll_down(&block_data.transactions, &appdata.schema, &state);

        // Decrement bix
        bix -= 1;
    }

    // Roll up the state and pool with remote blocks
    let mut is_valid = true;
    let mut block_info_prev = blockchain.get_block_info(bix_sync).await?;
    for block_data in blocks.iter() {
        // Validate the block
        if !block_data.block.validate(&block_data.transactions, 
                                      &block_info_prev, COMPLEXITY, 
                                      &appdata.schema, &state) {
            is_valid = false;
            break;
        }

        // Roll up state and pool
        state.roll_up(block_data.bix, &block_data.block, 
                      &block_data.transactions, &appdata.schema);
        pool.roll_up(&block_data.transactions, &appdata.schema, &state);

        // Change previous block info
        block_info_prev = block_data.get_block_info();
    }

    if is_valid {
        Ok(Some((state, pool)))
    } else {
        Ok(None)
    }
}


async fn migrate_blockchain(blocks: &[BlockData], 
                            blockchain: &Blockchain) -> TokioResult<()> {
    blockchain.truncate(blocks[0].bix - 1).await?;
    for block_data in blocks.iter() {
        blockchain.push_new_block(&block_data.block, 
                                  &block_data.transactions).await?;
    }
    Ok(())
}
