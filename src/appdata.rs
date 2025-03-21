use log::info;
use tokio::sync::RwLock;
use uqoin_core::utils::*;
use uqoin_core::pool::Pool;
use uqoin_core::state::State;
use uqoin_core::schema::Schema;
use uqoin_core::blockchain::Blockchain;

use crate::utils::*;
use crate::config::Config;


pub struct AppData {
    pub config: Config,
    pub schema: Schema,
    pub pool: RwLock<Pool>,
    pub state: RwLock<State>,
    pub blockchain: RwLock<Blockchain>,
    pub validators: RwLock<Vec<U256>>,
}


impl AppData {
    pub async fn new(config: Config) -> TokioResult<Self> {
        let schema = Schema::new();
        let pool = RwLock::new(Pool::new());
        let state = RwLock::new(State::new());
        let blockchain = RwLock::new(Blockchain::new(&config.data_path).await?);
        let validators = RwLock::new(config.validators.clone());

        let mut instance = Self {
            config, schema, pool, state, blockchain, validators
        };
        instance.initialize().await?;
        info!("AppData is ready");

        Ok(instance)
    }

    async fn initialize(&mut self) -> TokioResult<()> {
        // Evolve state through the blockchain
        let blockchain = self.blockchain.read().await;
        let mut state = self.state.write().await;
        let block_count = blockchain.get_block_count().await?;
        for bix in 1..=block_count {
            let block = blockchain.get_block(bix).await?;
            let transactions = 
                blockchain.get_transactions_of_block(&block).await?;
            state.roll_up(bix, &block, &transactions, &self.schema);
        }
        Ok(())
    }
}
