use serde::{Serialize, Deserialize};
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::utils::U256;

use crate::utils::*;


#[derive(Debug, Serialize, Deserialize)]
struct NodeInfo {
    wallet: U256,
}


/// Get list of known nodes.
async fn list_view(appdata: WebAppData) -> APIResult {
    let nodes = appdata.nodes.read().await.clone();
    Ok(HttpResponse::Ok().json(nodes))
}


/// Get node info.
async fn info_view(appdata: WebAppData) -> APIResult {
    let wallet = appdata.config.public_key.clone();
    let node_info = NodeInfo { wallet };
    Ok(HttpResponse::Ok().json(node_info))
}


pub fn load_scope() -> Scope {
    web::scope("/node")
        .route("/list", web::get().to(list_view))
        .route("/info", web::get().to(info_view))
}
