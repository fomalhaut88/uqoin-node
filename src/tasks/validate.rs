use std::sync::{Arc, Mutex, RwLock};

use log::info;
use uqoin_core::utils::U256;
use uqoin_core::block::Block;
use uqoin_core::transaction::Transaction;

use crate::utils::*;


const VALIDATE_TIMEOUT: u64 = 10000;
const NONCE_COUNT: usize = 100000;
const COMPLEXITY: usize = 24;
const THREADS: usize = 8;
const GROUPS_MAX: Option<usize> = None;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    // Input data for threads
    let block_hash_arc: Arc<RwLock<Option<U256>>> = 
        Arc::new(RwLock::new(None));
    let transactions_arc: Arc<RwLock<Option<Vec<Transaction>>>> = 
        Arc::new(RwLock::new(None));

    // Threads output
    let out_arc: Arc<Mutex<Option<(U256, Vec<Transaction>, [u8; 32])>>> = 
        Arc::new(Mutex::new(None));

    // Create threads
    for _ in 0..THREADS {
        // Copy arcs
        let block_hash_arc = Arc::clone(&block_hash_arc);
        let transactions_arc = Arc::clone(&transactions_arc);
        let out_arc = Arc::clone(&out_arc);

        // Spawn a thread
        std::thread::spawn(move || {
            // Random generator
            let mut rng = rand::rng();

            // Infinite loop
            loop {
                // Clone intermediate params
                let block_hash = block_hash_arc.read().unwrap().clone();
                let transactions = transactions_arc.read().unwrap().clone();
                let out = out_arc.lock().unwrap().clone();

                // If input is set and output is not set, run mining
                if out.is_none() && block_hash.is_some() 
                                 && transactions.is_some() {
                    // Mine nonce
                    let nonce = Block::mine(
                        &mut rng, &block_hash.clone().unwrap(), 
                        &U256::from_hex("1CC9E9F542DA6BADEF40919A1A4611E584DC607549C0775F5015A2B309793C15"), 
                        &transactions.clone().unwrap(), 
                        COMPLEXITY, Some(NONCE_COUNT)
                    );

                    // If nonce is mined, set `out`
                    if let Some(nonce) = nonce {
                        *out_arc.lock().unwrap() = Some(
                            (block_hash.unwrap(), transactions.unwrap(), nonce)
                        );
                    }
                } else {
                    // Wait for a while if params are not ready
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });
    }

    // Random generator
    let mut rng = rand::rng();

    // Infinite loop to process pool, state and threads
    loop {
        // Sleep
        tokio::time::sleep(
            tokio::time::Duration::from_millis(VALIDATE_TIMEOUT)
        ).await;

        // Update the pool and get ready transactions for the next block
        let (block_hash, transactions) = {
            // Get state and pool
            let state = appdata.state.read().await;
            let mut pool = appdata.pool.write().await;

            // Update pool transactions
            pool.update_groups(&appdata.schema, &state);

            // Extract transactions for a new block from pool
            let transactions = pool.prepare(&mut rng, &appdata.schema, &state, 
                &appdata.config.private_key, GROUPS_MAX);

            // Get last block hash
            let block_hash = state.get_last_block_info().hash.clone();

            // Return
            (block_hash, transactions)
        };

        // Update mining params if block hash or transactions changed
        if (*block_hash_arc.read().unwrap() != Some(block_hash.clone())) || (
                transactions_arc.read().unwrap().clone()
                    .unwrap_or(vec![]).len() != transactions.len()) {
            *block_hash_arc.write().unwrap() = Some(block_hash.clone());
            *transactions_arc.write().unwrap() = Some(transactions.clone());
        }

        // Check if nonce is mined
        let mut out = out_arc.lock().unwrap();
        if let Some((block_hash, transactions, nonce)) = out.as_ref() {
            // Get state for change below
            let mut state = appdata.state.write().await;
            let last_block_info = state.get_last_block_info();

            // If block hash is relevant
            if block_hash == &last_block_info.hash {
                // Create a new block
                // TODO: Set wallet from config
                let block = Block::build(
                    last_block_info.offset, block_hash.clone(), 
                    U256::from_hex("1CC9E9F542DA6BADEF40919A1A4611E584DC607549C0775F5015A2B309793C15"),
                    &transactions, U256::from_bytes(nonce), COMPLEXITY, 
                    &appdata.schema, &state
                );

                // If block built
                if let Some(block) = block {
                    // Get blockchain to change
                    let blockchain = appdata.blockchain.write().await;

                    // Push new block
                    let bix = blockchain.push_new_block(&block, 
                                                        &transactions).await?;

                    // Change state
                    state.roll_up(bix, &block, &transactions, &appdata.schema);

                    // Update pool
                    let mut pool = appdata.pool.write().await;
                    pool.update_groups(&appdata.schema, &state);

                    // Log
                    info!("New block added, bix = {}", bix);
                }
            }

            // Set back to `None`
            *out = None;
        }
    }
}
