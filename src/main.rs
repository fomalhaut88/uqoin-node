mod utils;
mod config;
mod appdata;
mod scopes;
mod tasks;

use actix_web::{get, web, App, HttpResponse, HttpServer};

use crate::config::Config;
use crate::appdata::AppData;
use crate::scopes::*;


#[get("/version")]
async fn version_view() -> HttpResponse {
    let version = env!("CARGO_PKG_VERSION");
    HttpResponse::Ok().body(version)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();

    let instance = AppData::new().await?;
    let appdata = web::Data::new(instance);

    // Background async task example (for syncing)
    actix_web::rt::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            println!("yes");
        }
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(appdata.clone())
            .service(version_view)
            .service(load_scope_client())
            .service(load_scope_blockchain())
            .service(load_resource_validator())
    })
        .workers(config.workers)
        .bind((config.host, config.port))?;

    server.run().await
}
