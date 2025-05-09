mod error;
mod utils;
mod config;
mod appdata;
mod scopes;
mod tasks;

use log::{info, error};
use serde::Serialize;
use tokio::io::{Result as TokioResult};
use actix_web::{get, web, App, HttpResponse, HttpServer};
use actix_web::middleware::Logger;
use actix_web::http::header;
use actix_cors::Cors;

use crate::utils::*;
use crate::config::Config;
use crate::appdata::AppData;
use crate::scopes::*;
use crate::tasks::*;


#[derive(Serialize)]
struct VersionInfo {
    version: String,
}


#[get("/version")]
async fn version_view() -> HttpResponse {
    let version = env!("CARGO_PKG_VERSION").to_string();
    HttpResponse::Ok().json(VersionInfo { version })
}


async fn run_task<F>(task: F, appdata: WebAppData) -> 
                     TokioResult<()> where 
                        F: AsyncFn(WebAppData) -> TokioResult<()> {
    loop {
        if let Err(err) = task(appdata.clone()).await {
            error!("{:?}", err);
            info!("Restarting task");
        }
    }
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
    if !appdata.config.lite_mode {
        actix_web::rt::spawn(run_task(mine_task, appdata.clone()));
    }
    actix_web::rt::spawn(run_task(sync_task, appdata.clone()));

    // Create API server
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_header(header::CONTENT_TYPE);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(appdata.clone())
            .service(version_view)
            .service(load_scope_coin())
            .service(load_scope_client())
            .service(load_scope_blockchain())
            .service(load_scope_node())
    })
        .workers(workers)
        .bind((host, port))?;

    // Run API server
    server.run().await
}
