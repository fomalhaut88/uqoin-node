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
        Ok(Self { config, schema, pool, state, blockchain, validators })
    }
}
