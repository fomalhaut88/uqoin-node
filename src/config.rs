use std::env;

use uqoin_core::utils::*;


pub struct Config {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub data_path: String,
    pub validators: Vec<U256>,
    pub private_key: U256,
}


impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or("localhost".to_string()),
            port: env::var("PORT").unwrap_or("8080".to_string())
                                  .parse().unwrap(),
            workers: env::var("WORKERS").unwrap_or("1".to_string())
                                        .parse().unwrap(),
            data_path: env::var("DATA_PATH").unwrap_or("./tmp/db".to_string()),
            validators: Vec::new(),
            // TODO: Make private key from env as required.
            private_key: env::var("PRIVATE_KEY").map(|s| U256::from_hex(&s))
                                                .unwrap_or(U256::from_hex("07D9FE88AF04AE8C9C17071D8F07DDEE7B62B107A95AC26DF4D8610705B67456")),
        }
    }
}
