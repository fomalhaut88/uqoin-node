use std::env;

use lbasedb::path_concat;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::coin::coin_order_by_symbol;


/// Config parameters.
pub struct Config {
    /// Host to deploy.
    pub host: String,

    /// Port to deploy.
    pub port: u16,

    /// Number of workers in HTTP server.
    pub workers: usize,

    /// Path to blockchain data
    pub data_path: String,

    /// Remote nodes to sync.
    pub nodes: Vec<String>,

    /// Validator private key.
    pub private_key: Option<U256>,

    /// Validator public key.
    pub public_key: Option<U256>,

    /// Threads in mining.
    pub mining_threads: usize,

    /// Minimum fee allowed (as order).
    pub fee_min_order: u64,

    /// Node sync timeout.
    pub node_sync_timeout: u64,

    /// Maximum blocks to sync.
    pub node_sync_block_count: u64,

    /// Mining timeout.
    pub mining_timeout: u64,

    /// Mining update count.
    pub mining_update_count: u64,

    /// Mining nonce count per iteration.
    pub mining_nonce_count_per_iteration: usize,

    /// Maximum groups from the pool to mine into a block.
    pub mining_groups_max: Option<usize>,

    /// Lite mode
    pub lite_mode: bool,
}


impl Config {
    pub fn from_env() -> Self {
        let schema = Schema::new();

        let private_key = env::var("PRIVATE_KEY")
            .map(|s| U256::from_hex(&s)).ok();
        let public_key = private_key.as_ref()
            .map(|key| schema.get_public(&key));

        let nodes: Vec<String> = env::var("NODES")
            .map(|l| l.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or(vec![]);

        let data_path = env::var("DATA_PATH").unwrap_or("./tmp".to_string());

        let lite_mode = private_key.is_none();

        std::fs::create_dir_all(&data_path).unwrap();

        Self {
            host: env::var("HOST").unwrap_or("localhost".to_string()),
            port: env::var("PORT").unwrap_or("5772".to_string())
                                  .parse().unwrap(),
            workers: env::var("WORKERS").unwrap_or("1".to_string())
                                        .parse().unwrap(),
            data_path, nodes, private_key, public_key,
            mining_threads: env::var("MINING_THREADS")
                                .unwrap_or("1".to_string()).parse().unwrap(),
            fee_min_order: env::var("FEE_MIN")
                               .map(|s| coin_order_by_symbol(&s)).unwrap_or(0),
            node_sync_timeout: env::var("NODE_SYNC_TIMEOUT")
                                   .map(|s| s.parse().unwrap()).unwrap_or(5000),
            node_sync_block_count: env::var("NODE_SYNC_BLOCK_COUNT")
                                   .map(|s| s.parse().unwrap()).unwrap_or(1000),
            mining_timeout: env::var("MINING_TIMEOUT")
                                .map(|s| s.parse().unwrap()).unwrap_or(20000),
            mining_update_count: env::var("MINING_UPDATE_COUNT")
                                 .map(|s| s.parse().unwrap()).unwrap_or(20),
            mining_nonce_count_per_iteration: 
                env::var("MINING_NONCE_COUNT_PER_ITERATION")
                    .map(|s| s.parse().unwrap()).unwrap_or(100000),
            mining_groups_max: env::var("MINING_GROUPS_MAX")
                    .map(|s| s.parse().unwrap()).ok(),
            lite_mode,
        }
    }

    pub fn get_state_path(&self) -> String {
        path_concat!(self.data_path.clone(), "state.json")
    }

    pub fn get_mining_validate_iter_timeout(&self) -> u64 {
        self.mining_timeout / self.mining_update_count
    }
}
