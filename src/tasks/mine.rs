use std::sync::{Arc, RwLock};

use rand::Rng;
use log::{info, warn};
use tokio::time::{sleep, Duration};
use uqoin_core::utils::U256;
use uqoin_core::block::{Block, COMPLEXITY};
use uqoin_core::transaction::Transaction;

use crate::utils::*;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    // Input data for threads
    let block_hash_arc: Arc<RwLock<Option<U256>>> = 
        Arc::new(RwLock::new(None));
    let transactions_arc: Arc<RwLock<Option<Vec<Transaction>>>> = 
        Arc::new(RwLock::new(None));

    // Threads output
    let out_arc: Arc<RwLock<Option<(U256, Vec<Transaction>, [u8; 32])>>> = 
        Arc::new(RwLock::new(None));

    // Create threads
    for _ in 0..appdata.config.mining_threads {
        // Copy arcs
        let block_hash_arc = Arc::clone(&block_hash_arc);
        let transactions_arc = Arc::clone(&transactions_arc);
        let out_arc = Arc::clone(&out_arc);
        let public_key = appdata.config.public_key.clone().unwrap();
        let mining_nonce_count_per_iteration = 
            appdata.config.mining_nonce_count_per_iteration;

        // Spawn a thread
        std::thread::spawn(move || {
            // Random generator
            let mut rng = rand::rng();

            // Infinite loop
            loop {
                // Clone intermediate params
                let block_hash = block_hash_arc.read().unwrap().clone();
                let transactions = transactions_arc.read().unwrap().clone();
                let out = out_arc.read().unwrap().clone();

                // If input is set and output is not set, run mining
                if block_hash.is_some() && transactions.is_some() {
                    if out.is_none() || (out.unwrap().1.len() 
                                      < transactions.as_ref().unwrap().len()) {
                        // Mine nonce
                        let nonce = Block::mine(
                            &mut rng, block_hash.as_ref().unwrap(), &public_key,
                            transactions.as_ref().unwrap(), COMPLEXITY, 
                            Some(mining_nonce_count_per_iteration)
                        );

                        // If nonce is mined, set `out`
                        if let Some(nonce) = nonce {
                            *out_arc.write().unwrap() = Some((
                                block_hash.unwrap(), 
                                transactions.unwrap(), 
                                nonce,
                            ));
                        }

                        // Continue the loop
                        continue;
                    }
                }

                // Wait for a while if params are not ready
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
    }

    // Random generator
    let mut rng = rand::rng();

    // Infinite loop to process pool, state and threads
    loop {
        // Try to update transactions to join `MINING_UPDATE_COUNT` times with  
        // the sleepage `MINING_TIMEOUT`.
        for _ in 0..appdata.config.mining_update_count {
            // Get ready transactions for the next block
            let (block_hash, transactions) = 
                get_transactions_from_pool(&mut rng, &appdata).await;

            // Update mining params if block hash or transactions changed
            if (*block_hash_arc.read().unwrap() != Some(block_hash.clone())) 
                    || (transactions_arc.read().unwrap().as_ref()
                        .unwrap_or(&vec![]).len() < transactions.len()) {
                *block_hash_arc.write().unwrap() = Some(block_hash.clone());
                *transactions_arc.write().unwrap() = Some(transactions.clone());
            }

            // Sleep
            sleep(Duration::from_millis(
                appdata.config.get_mining_validate_iter_timeout()
            )).await;
        }

        // Check if nonce is mined
        let mut out = out_arc.write().unwrap();
        if let Some((block_hash, transactions, nonce)) = out.as_ref() {
            // Add new block
            add_new_block(block_hash, transactions, nonce, &appdata).await?;

            // Set back to `None`
            *out = None;
        }
    }
}


async fn get_transactions_from_pool<R: Rng>(
        rng: &mut R, appdata: &WebAppData) -> (U256, Vec<Transaction>) {
    // Get state and pool
    let state = appdata.state.read().await;
    let pool = appdata.pool.read().await;

    // Extract transactions for a new block from pool
    // TODO: Pass senders from pool.prepare to add_new_block so it will increase
    // the performance.
    let (transactions, _) = pool.prepare(
        rng, &state, &appdata.schema, 
        &appdata.config.private_key.as_ref().unwrap(), 
        appdata.config.mining_groups_max
    );

    // Get last block hash
    let block_hash = state.get_last_block_info().hash.clone();

    // Return
    (block_hash, transactions)
}


async fn add_new_block(block_hash: &U256, transactions: &[Transaction], 
                       nonce: &[u8; 32], appdata: &WebAppData) -> 
                       std::io::Result<()> {
    // Lock blockchain to change
    let blockchain = appdata.blockchain.write().await;

    // Lock state for change below
    let mut state = appdata.state.write().await;
    let last_block_info = state.get_last_block_info();

    // If block hash is relevant
    if block_hash == &last_block_info.hash {
        // Calculate senders
        let senders = Transaction::calc_senders(&transactions, &state, 
                                                &appdata.schema);

        // Create a new block
        let block = Block::build(
            last_block_info, appdata.config.public_key.clone().unwrap(),
            transactions, U256::from_bytes(nonce), COMPLEXITY, &state, &senders
        );

        match block {
            Ok(block) => {
                // Push new block
                let bix = blockchain.push_new_block(&block, transactions).await?;

                // Change state
                state.roll_up(bix, &block, transactions, &appdata.schema);

                // Update pool
                let mut pool = appdata.pool.write().await;
                pool.update(&state, &appdata.schema);

                // Dump state
                state.dump(&appdata.config.get_state_path()).await?;

                // Log
                info!("New block added, bix = {}", bix);
            },
            Err(err) => {
                warn!("Unable to build a block: {:?}", err);
                info!("Clearing pool...");
                appdata.pool.write().await.clear();
                // It may affect on mining threads because they can mine
                // old transactions without a stop.
            },
        }
    } else {
        info!("Could not add block, hashes diverge");
    }

    Ok(())
}
