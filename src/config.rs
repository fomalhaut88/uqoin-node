use std::env;

use lbasedb::path_concat;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;


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
    pub private_key: U256,

    /// Validator public key.
    pub public_key: U256,

    /// Threads in mining.
    pub mining_threads: usize,
}


impl Config {
    pub fn from_env() -> Self {
        let schema = Schema::new();

        let private_key = env::var("PRIVATE_KEY").map(|s| U256::from_hex(&s))
                              .expect("Environment PRIVATE_KEY is required");
        let public_key = schema.get_public(&private_key);

        let nodes: Vec<String> = env::var("NODES")
            .map(|l| l.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or(Vec::new());

        let data_path = env::var("DATA_PATH").unwrap_or("./tmp".to_string());

        std::fs::create_dir_all(&data_path).unwrap();

        Self {
            host: env::var("HOST").unwrap_or("localhost".to_string()),
            port: env::var("PORT").unwrap_or("8080".to_string())
                                  .parse().unwrap(),
            workers: env::var("WORKERS").unwrap_or("1".to_string())
                                        .parse().unwrap(),
            data_path, nodes, private_key, public_key,
            mining_threads: env::var("MINING_THREADS")
                                .unwrap_or("1".to_string()).parse().unwrap()
        }
    }

    pub fn get_state_path(&self) -> String {
        path_concat!(self.data_path.clone(), "state.json")
    }
}
