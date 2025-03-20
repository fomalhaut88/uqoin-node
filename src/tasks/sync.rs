use tokio::time::{sleep, Duration};

use crate::utils::*;


const SYNC_TIMEOUT: u64 = 2000;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    // use log::info;
    loop {
        sleep(Duration::from_millis(SYNC_TIMEOUT)).await;
        // info!("sync: {}", appdata.config.workers);
    }
}
