use serde::Serialize;
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::utils::U256;
use uqoin_core::coin::coin_symbol;

use crate::utils::*;


#[derive(Debug, Serialize)]
struct NodeInfo {
    wallet: Option<U256>,
    fee: Option<String>,
    free_split: bool,
    lite_mode: bool,
}


/// Get list of known nodes.
async fn list_view(appdata: WebAppData) -> APIResult {
    let nodes = appdata.nodes.read().await.clone();
    Ok(HttpResponse::Ok().json(nodes))
}


/// Get node info.
async fn info_view(appdata: WebAppData) -> APIResult {
    let wallet = appdata.config.public_key.clone();
    let fee = if appdata.config.fee_min_order > 0 {
        let symbol = coin_symbol(appdata.config.fee_min_order);
        Some(symbol)
    } else {
        None
    };
    let free_split = appdata.config.free_split;
    let lite_mode = appdata.config.lite_mode;
    let node_info = NodeInfo { wallet, fee, free_split, lite_mode };
    Ok(HttpResponse::Ok().json(node_info))
}


pub fn load_scope() -> Scope {
    web::scope("/node")
        .route("/list", web::get().to(list_view))
        .route("/info", web::get().to(info_view))
}
