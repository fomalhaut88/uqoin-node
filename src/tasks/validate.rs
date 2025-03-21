use log::info;
use uqoin_core::utils::U256;
use uqoin_core::block::Block;

use crate::utils::*;


// const VALIDATE_TIMEOUT: u64 = 3000;
const NONCE_COUNT: usize = 100000;
const COMPLEXITY: usize = 24;
// const THREADS: usize = 8;
const GROUPS_MAX: usize = 100;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    let mut rng = rand::rng();

    loop {
        // TODO: Implement sleepage so the transactions have time to come

        // Update the pool and get ready transactions for the next block
        let transactions = {
            let mut pool = appdata.pool.write().await;
            let state = appdata.state.read().await;
            pool.update_groups(&appdata.schema, &state);
            pool.prepare(&mut rng, &appdata.schema, &state, &appdata.config.private_key, GROUPS_MAX)
        };

        // Try to mine a block
        let block_hash = {
            let state = appdata.state.read().await;
            state.get_last_block_info().hash.clone()
        };
        // TODO: Set wallet from config
        // TODO: Implement multithreading
        let nonce = Block::mine(&mut rng, &block_hash, 
                                &U256::from_hex("1CC9E9F542DA6BADEF40919A1A4611E584DC607549C0775F5015A2B309793C15"), 
                                &transactions, COMPLEXITY, Some(NONCE_COUNT));

        // If nonce found
        if let Some(nonce) = nonce {
            // Get state for change
            let mut state = appdata.state.write().await;
            let last_block_info = state.get_last_block_info();

            // If block hash is relevant
            if block_hash == last_block_info.hash {
                // Create a new block
                // TODO: Set wallet from config
                let block = Block::build(
                    last_block_info.offset, block_hash, U256::from_hex("1CC9E9F542DA6BADEF40919A1A4611E584DC607549C0775F5015A2B309793C15"),
                    &transactions, U256::from_bytes(&nonce), COMPLEXITY, &appdata.schema, &state
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
        }
    }
}
