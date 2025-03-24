use serde::{Serialize, Deserialize};
use actix_web::{web, HttpResponse, Scope};
use uqoin_core::utils::U256;

use crate::utils::*;


#[derive(Deserialize)]
struct Query {
    coin: String,
}


#[derive(Serialize)]
struct WalletInfo {
    wallet: String,
}


/// Get coin info.
async fn info_view(appdata: WebAppData, 
                   query: web::Query<Query>) -> APIResult {
    let coin = U256::from_hex(&query.coin);
    let state = appdata.state.read().await;
    if let Some(coin_info) = state.get_coin_info(&coin) {
        Ok(HttpResponse::Ok().json(coin_info))
    } else {
        Ok(HttpResponse::Ok().body("{}"))
    }
}


/// Get coin owner.
async fn owner_view(appdata: WebAppData, 
                    query: web::Query<Query>) -> APIResult {
    let coin = U256::from_hex(&query.coin);
    let state = appdata.state.read().await;
    if let Some(owner) = state.get_owner(&coin) {
        Ok(HttpResponse::Ok().json(WalletInfo { wallet: owner.to_hex() }))
    } else {
        Ok(HttpResponse::Ok().body("{}"))
    }
}


pub fn load_scope() -> Scope {
    web::scope("/coin")
        .route("/info", web::get().to(info_view))
        .route("/owner", web::get().to(owner_view))
}
