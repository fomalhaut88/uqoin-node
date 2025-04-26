use log::{info, error};
use rand::prelude::IndexedRandom;
use tokio::io::{Error, ErrorKind};
use tokio::time::{sleep, Duration};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uqoin_core::block::{BlockInfo, BlockData, COMPLEXITY};
use uqoin_core::blockchain::Blockchain;
use uqoin_core::state::State;
use uqoin_core::transaction::{Transaction, Group, group_transactions};

use crate::async_try_many;
use crate::utils::*;
use crate::scopes::blockchain::{BlockQuery, BlockManyQuery};


const TRY_NODE_ATTEMPTS: usize = 10;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    // Random generator
    let mut rng = rand::rng();

    loop {
        // Sync timeout
        sleep(Duration::from_millis(appdata.config.node_sync_timeout)).await;

        // Choose a random node
        if let Some(random_node) = appdata.nodes.read().await.choose(&mut rng) {
            info!("Trying to sync with {}", random_node);

            // Request last block info of the node
            if let Ok(last_info_remote) = request_node::<BlockInfo, _>(
                    &random_node, "/blockchain/block-info", 
                    None::<BlockQuery>).await {
                // Get local last block info
                let last_info_local: BlockInfo = appdata.state.read().await
                                                .get_last_block_info().clone();

                // Sync basic condition (remote transaction count is greater
                // than the local one)
                if last_info_remote.offset > last_info_local.offset {
                    info!("Need to sync with {}", random_node);

                    // Request for sync point
                    let bix_sync = request_for_divergent_bix(
                        std::cmp::min(last_info_remote.bix, last_info_local.bix),
                        &random_node, &*appdata.blockchain.read().await
                    ).await?;

                    info!("Need to sync after bix = {}", bix_sync);

                    // Limit the block count to sync
                    let bix_until = std::cmp::min(
                        last_info_remote.bix, 
                        bix_sync + appdata.config.node_sync_block_count
                    );

                    // Set syncing if there are too many blocks forward to sync
                    if bix_until < last_info_remote.bix {
                        if !*appdata.is_syncing.read().await {
                            *appdata.is_syncing.write().await = true;
                        }
                    }

                    // Request for remote blocks
                    let blocks = request_for_remote_blocks(
                        bix_sync + 1, bix_until, &random_node
                    ).await?;

                    info!("Got {} blocks to roll up", blocks.len());

                    // Check divergent blocks
                    if let Some((state_new, trs_vec)) = 
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

                        // Update pool
                        for trs in trs_vec.into_iter() {
                            let senders = Transaction::calc_senders(
                                &trs, &state, &appdata.schema
                            );
                            if let Ok(group) = Group::new(trs, &state, 
                                                          &senders) {
                                pool.add(group, senders[0].clone());
                            }
                        }
                        pool.update(&state, &appdata.schema);

                        // Dump state
                        state.dump(&appdata.config.get_state_path()).await?;

                        // Set is_syncing to `false` if everything is up to date
                        if bix_until == last_info_remote.bix {
                            if *appdata.is_syncing.read().await {
                                *appdata.is_syncing.write().await = false;
                            }
                        }

                        info!("Synced with {} successfully", random_node);
                    } else {
                        info!("Blocks are invalid in {}", random_node);
                    }
                } else {
                    info!("No need to sync with {}", random_node);
                }
            } else {
                info!("Cound not reach the node {}", random_node);
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

    info!("External node request: {}", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5)).build().unwrap();

    let resp = client.get(&url).send().await
                     .map_err(|_| Error::new(ErrorKind::NotFound, url))?;

    let content: String = resp.text().await
        .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
    let instance = serde_json::from_str::<T>(&content)?;
    Ok(instance)
}


async fn request_for_divergent_bix(bix_last: u64, node: &str, 
                                   blockchain: &Blockchain) -> 
                                   TokioResult<u64> {
    find_divergence(bix_last, async |bix| {
        // Get remote block info
        let block_info: BlockInfo = async_try_many!(
            TRY_NODE_ATTEMPTS, request_node, 
            node, "/blockchain/block-info",
            Some(BlockQuery { bix: Some(bix) })
        )?;

        // Get local block hash for the `bix`
        let hash_local = blockchain.get_block(bix).await?.hash;

        // Check if hashes are equal
        Ok(block_info.hash == hash_local)
    }).await
}


async fn request_for_remote_blocks(bix_from: u64, bix_to: u64, node: &str) -> 
                                   TokioResult<Vec<BlockData>> {
    async_try_many!(
        TRY_NODE_ATTEMPTS, request_node, 
        &node, "/blockchain/block-many",
        Some(BlockManyQuery { bix: bix_from, count: bix_to + 1 - bix_from })
    )
}


async fn check_divergent_blocks(blocks: &[BlockData], appdata: &WebAppData) -> 
                                TokioResult<Option<(State, Vec<Vec<Transaction>>)>> {
    // Get blockchain and clone the current state and pool
    let blockchain = appdata.blockchain.read().await;
    let mut state = appdata.state.read().await.clone();
    let mut trs_vec = Vec::new();
    // let mut pool = appdata.pool.read().await.clone();

    let bix_sync = blocks[0].bix - 1;

    let mut bix = blockchain.get_block_count().await?;

    // Roll down the state and pool with local blocks
    while bix > bix_sync {
        // Get local block data
        let block_data = blockchain.get_block_data(bix).await?;

        // Roll back state
        state.roll_down(bix, &block_data.block, &block_data.transactions, 
                        &appdata.schema);

        // // Roll back pool
        // let senders = Transaction::calc_senders(&block_data.transactions, 
        //                                         &state, &appdata.schema);
        // pool.roll_down(&block_data.transactions, &state, &senders);
        // pool.update(&state);

        // Calculate senders
        let senders = Transaction::calc_senders(&block_data.transactions, 
                                                &state, &appdata.schema);

        // Collect rolled down groups of transactions
        for (_, group, _) in group_transactions(block_data.transactions, &state, 
                                                &senders) {
            let trs = group.transactions().to_vec();
            trs_vec.push(trs);
        }

        // Decrement bix
        bix -= 1;
    }

    // Roll up the state and pool with remote blocks
    let mut is_valid = true;
    let mut block_info_prev = blockchain.get_block_info(bix_sync).await?;
    for block_data in blocks.iter() {
        // Calculate senders
        let senders = Transaction::calc_senders(&block_data.transactions, 
                                                &state, &appdata.schema);

        // Validate the block
        let validation_result = block_data.block.validate(
            &block_data.transactions, &block_info_prev, COMPLEXITY, 
            &state, &senders
        );

        if let Err(err) = validation_result {
            error!("{}", err);
            is_valid = false;
            break;
        }

        // Roll up state
        state.roll_up(block_data.bix, &block_data.block, 
                      &block_data.transactions, &appdata.schema);
        // pool.roll_up(&block_data.transactions, &state);
        // pool.update(&state);

        // Change previous block info
        block_info_prev = block_data.get_block_info();
    }

    if is_valid {
        Ok(Some((state, trs_vec)))
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
