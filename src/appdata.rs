use tokio::io::Result as TokioResult;


pub struct AppData {}


impl AppData {
    pub async fn new() -> TokioResult<Self> {
        Ok(Self {})
    }
}
