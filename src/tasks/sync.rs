use tokio::time::{sleep, Duration};

use crate::utils::*;


const SYNC_TIMEOUT: u64 = 2000;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    loop {
        sleep(Duration::from_millis(SYNC_TIMEOUT)).await;
        println!("sync: {}", appdata.config.workers);
    }
}
