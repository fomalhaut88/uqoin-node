use std::env;


pub struct Config {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    // pub data_path: String,
}


impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or("localhost".to_string()),
            port: env::var("PORT").unwrap_or("8080".to_string())
                                  .parse().unwrap(),
            workers: env::var("WORKERS").unwrap_or("1".to_string())
                                        .parse().unwrap(),
            // data_path: env::var("DATA_PATH").unwrap_or("./tmp/db".to_string()),
        }
    }
}
