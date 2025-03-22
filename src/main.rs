mod utils;
mod config;
mod appdata;
mod scopes;
mod tasks;

use actix_web::{get, web, App, HttpResponse, HttpServer};
use actix_web::middleware::Logger;

use crate::utils::*;
use crate::config::Config;
use crate::appdata::AppData;
use crate::scopes::*;
use crate::tasks::*;


#[get("/version")]
async fn version_view() -> HttpResponse {
    let version = env!("CARGO_PKG_VERSION");
    HttpResponse::Ok().body(version)
}


#[actix_web::main]
async fn main() -> TokioResult<()> {
    // Config
    let config = Config::from_env();

    // Initialize logging
    let env = env_logger::Env::new().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    // Run options
    let workers = config.workers;
    let host = config.host.clone();
    let port = config.port;

    // Create appdata instance
    let instance = AppData::new(config).await?;
    let appdata = web::Data::new(instance);

    // Background tasks
    actix_web::rt::spawn(mine_task(appdata.clone()));
    // actix_web::rt::spawn(sync_task(appdata.clone()));

    // Create API server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(appdata.clone())
            .service(version_view)
            .service(load_scope_client())
            .service(load_scope_blockchain())
            .service(load_scope_validator())
    })
        .workers(workers)
        .bind((host, port))?;

    // Run API server
    server.run().await
}
